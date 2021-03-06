use crate::{buffer_vec::BufferVec, FragmentId, GlobalRenderResources};
use common::AsBytes;
use periodic_table::Element;
use std::mem::{self, MaybeUninit};
use ultraviolet::Vec3;

/// Packed bit field
/// | 0 .. 7 | ----------- | 7 .. 31 |
///   ^ atomic number - 1    ^ unspecified
///
/// TODO: Try using a buffer as an atom radius lookup table.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AtomKind(u32);
impl AtomKind {
    pub fn new(element: Element) -> Self {
        Self(((element as u8 - 1) & 0b1111_111) as u32)
    }

    pub fn element(&self) -> Element {
        let n = (self.0 & 0b1111_111) as u8 + 1;
        Element::from_atomic_number(n)
            .unwrap_or_else(|| unreachable!("invalid atomic number in atom kind"))
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct AtomRepr {
    pub pos: Vec3, // with respect to fragment center
    pub kind: AtomKind,
}

static_assertions::const_assert_eq!(mem::size_of::<AtomRepr>(), 16);
unsafe impl AsBytes for AtomRepr {}

/// Essentially a per-fragment uniform.
#[repr(C, align(16))]
struct AtomBufferHeader {
    fragment_id: FragmentId, // 64 bits
}

static_assertions::const_assert_eq!(mem::size_of::<FragmentId>(), 8);
static_assertions::const_assert_eq!(mem::size_of::<AtomBufferHeader>(), 16);
unsafe impl AsBytes for AtomBufferHeader {}

pub struct Atoms {
    bind_group: wgpu::BindGroup,
    buffer: BufferVec<AtomBufferHeader, AtomRepr>,
    number_of_atoms: usize,
}

impl Atoms {
    pub fn new<I>(gpu_resources: &GlobalRenderResources, fragment_id: FragmentId, iter: I) -> Self
    where
        I: IntoIterator<Item = AtomRepr>,
        I::IntoIter: ExactSizeIterator,
    {
        let atoms = iter.into_iter();
        let number_of_atoms = atoms.len();

        assert!(number_of_atoms > 0, "must have at least one atom");

        let buffer = BufferVec::new_with_data(
            &gpu_resources.device,
            wgpu::BufferUsage::STORAGE,
            number_of_atoms as u64,
            |header, array| {
                // header.write(AtomBufferHeader { fragment_id });
                *header = MaybeUninit::new(AtomBufferHeader { fragment_id });

                for (block, atom) in array.iter_mut().zip(atoms) {
                    // block.write(atom);
                    *block = MaybeUninit::new(atom);
                }
            },
        );

        let bind_group = gpu_resources
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &gpu_resources.atom_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: buffer.inner_buffer(),
                        offset: 0,
                        size: None,
                    },
                }],
            });

        Self {
            bind_group,
            buffer,
            number_of_atoms,
        }
    }

    pub fn copy_new(
        &self,
        render_resources: &GlobalRenderResources,
        fragment_id: FragmentId,
    ) -> Self {
        let buffer = self.buffer.copy_new(render_resources, false);

        render_resources.queue.write_buffer(
            buffer.inner_buffer(),
            0,
            AtomBufferHeader { fragment_id }.as_bytes(),
        );

        let bind_group = render_resources
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_resources.atom_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: buffer.inner_buffer(),
                        offset: 0,
                        size: None,
                    },
                }],
            });

        Self {
            bind_group,
            buffer,
            number_of_atoms: self.number_of_atoms,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn len(&self) -> usize {
        self.number_of_atoms
    }
}
