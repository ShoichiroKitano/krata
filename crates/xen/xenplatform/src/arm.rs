use crate::{
    boot::{BootDomain, BootSetupPlatform, DomainSegment},
    error::{Error, Result},
    sys::{XEN_PAGE_SHIFT},
};
use xencall::sys::{
    CreateDomain,
    XEN_DOMCTL_CDF_IOMMU,
};
use std::comp;

#[derive(Default, Clone)]
pub struct ArmPlatform {
    rambank_size [u64; 2],
}

const ARM_PAGE_SHIFT: u64 = 12;
const ARM_PAGE_SIZE: u64 = 1 << ARM_PAGE_SHIFT;
const ARM_GUEST_RAM0_BASE: u64 = 0x40000000;
const ARM_GUEST_RAM0_SIZE: u64 = 0xc0000000;
const ARM_GUEST_RAM1_BASE: u64 = 0x0200000000;
const ARM_GUEST_RAM1_SIZE: u64 = 0xfe00000000;
const ARM_LPAE_SHIFT: u64 = 9;
const ARM_PFN_4K_SHIFT: u64 = 0;
const ARM_PFN_2M_SHIFT: u64 = ARM_PFN_4K_SHIFT + ARM_LPAE_SHIFT;
const ARM_PFN_1G_SHIFT: u64 = ARM_PFN_2M_SHIFT + ARM_LPAE_SHIFT;
const ARM_PFN_512G_SHIFT: u64 = ARM_PFN_1G_SHIFT + ARM_LPAE_SHIFT;


impl ArmPlatform {
    pub fn new() -> Self {
        Self {
            rambank_size: [0, 0]
        }
    }
}

#[async_trait::async_trait]
impl BootSetupPlatform for ArmPlatform {
    fn create_domain(&self, enable_iommu: bool) -> CreateDomain {
        CreateDomain {
            flags: if enable_iommu {
                XEN_DOMCTL_CDF_IOMMU
            } else {
                0
            },
            ..Default::default()
        }
    }

    fn page_size(&self) -> u64 {
        ARM_PAGE_SIZE
    }

    fn page_shift(&self) -> u64 {
        ARM_PAGE_SHIFT
    }

    fn needs_early_kernel(&self) -> bool {
        false
    }

    fn hvm(&self) -> bool {
        false
    }

    async fn initialize_early(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn initialize_memory(&mut self, domain: &mut BootDomain) -> Result<()> {
        domain.call.set_address_size(domain.domid, 64).await?;
        let mut ramsize = domain.total_pages << XEN_PAGE_SHIFT
        let bankmax = [ARM_GUEST_RAM0_SIZE, ARM_GUEST_RAM1_SIZE];
        for i in 0..2 {
            let banksize: u64 = if ramsize > bankmax[i] {
                bankmax[i]
            } else {
                ramsize
            }
            ramsize -= banksize;
            self.rambank_size[i] = banksize >> XEN_PAGE_SHIFT;
        }
        let bankbase = [ARM_GUEST_RAM0_BASE, ARM_GUEST_RAM1_BASE];
        for i in 0..2 {
            if self.rambank_size[i] == 0 {
                break
            }
            if let Err(e) = self.populate_guest_memory(domain, bankbase[i] >> XEN_PAGE_SHIFT, self.rambank_size[i]) {
                return Err(e)
            }
        }

        //TODO
        let ramdisk_size = TODO!
        let dtb_size = TODO!
        let modsize = dtb_size + ramdisk_size
        let mut modbase: u64;
        let bank0end = bankbase[0] + (rambank_size[0] << XEN_PAGE_SHIFT);
        let ram128mb = ARM_GUEST_RAM0_BASE + (128<<20)
        if bank0end >= ram128mb + modsize && domain.image_info.virt_kstart < ram128mb {
            modbase = ram128mb;
        } else if ( bank0end - modsize > kernend ) {
            modbase = bank0end - modsize;
        } else if (kernbase - bankbase[0] > modsize ) {
            modbase = kernbase - modsize;
        } else {
            TODO!
        }

        if ramdisk_size != 0 {
            dom->modules[0].seg.vstart = modbase;
            dom->modules[0].seg.vend = modbase + ramdisk_size;
            modbase += ramdisk_size;
        }

        if dtb_size != 0 {
            dom->devicetree_seg.vstart = modbase;
            dom->devicetree_seg.vend = modbase + dtb_size;
            modbase += dtb_size;
        }
        Ok(())
    }

    async fn alloc_page_tables(&mut self, domain: &mut BootDomain) -> Result<Option<DomainSegment>> {
        todo!()
    }

    async fn alloc_p2m_segment(&mut self, domain: &mut BootDomain) -> Result<Option<DomainSegment>> {
        todo!()
    }

    async fn alloc_magic_pages(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn setup_page_tables(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn setup_shared_info(
        &mut self,
        domain: &mut BootDomain,
        shared_info_frame: u64,
    ) -> Result<()> {
        Ok(())
    }

    async fn setup_start_info(
        &mut self,
        domain: &mut BootDomain,
        shared_info_frame: u64,
    ) -> Result<()> {
        Ok(())
    }

    async fn bootlate(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn gnttab_seed(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn vcpu(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn setup_hypercall_page(&mut self, domain: &mut BootDomain) -> Result<()> {
        Ok(())
    }

    async fn populate_guest_memory(
        &mut self,
        domain: &mut BootDomain,
        base_pfn: u64,
        nr_pfns: u64
        ) -> Result<()> {
        let mut pfn = 0u64;

        while(pfn < nr_pfns) {
            let mut allocsz = comp::min(int, 1024*1024, nr_pfns - pfn);
            match self.populate_one_size(domid, ARM_PFN_512G_SHIFT, base_pfn + pfn, &mut allocsz) {
                Ok(v) => if v > 0 {
                    pfn += allocsz;
                    continue
                },
                Err(e) => return Err(e),
            }
            match self.populate_one_size(domid, ARM_PFN_1G_SHIFT, base_pfn + pfn, &mut allocsz) {
                Ok(v) => if v > 0 {
                    pfn += allocsz;
                    continue
                },
                Err(e) => return Err(e),
            }

            match self.populate_one_size(domid, ARM_PFN_2M_SHIFT, base_pfn + pfn, &mut allocsz) {
                Ok(v) => if v > 0 {
                    pfn += allocsz;
                    continue
                },
                Err(e) => return Err(e),
            }

            match self.populate_one_size(domid, ARM_PFN_4K_SHIFT, base_pfn + pfn, &mut allocsz) {
                Ok(0) => return Err(Error::PopulatePhysmapFailed),
                Err(e) => return Err(e),
                _ => { pfn += allocsz; },
            }
        }
        Ok(())
    }

    async fn populate_one_size(
        &mut self,
        domain: &mut BootDomain
        pfn_shift: u8,
        base_pfn: u64,
        nr_pfns: &mut u64
        ) -> Result<usize> {
        let next_shift = pfn_shift + ARM_LPAE_SHIFT;
        let next_mask: u64 = (1 << next_shift) - 1;

        let mut end_pfn = base_pfn + *nr_pfns;

        if pfn_shift == PFN_512G_SHIFT {
            return Ok(0);
        }

        let mask: u64 = (1 << pfn_shift) - 1;
        if mask & base_pfn {
            return Ok(0);
        }

        let next_boundary: u64 = (base_pfn + (1 << next_shift)) & ~next_mask;
        if (base_pfn & next_mask) && end_pfn > next_boundary {
            end_pfn = next_boundary;
        }

        let count = ( end_pfn - base_pfn ) >> pfn_shift;

        if count == 0 {
            return Ok(0);
        }

        let mut extents = [0u64; count as usize];
        for i in 0..count {
            extents[i] = base_pfn + (i<<pfn_shift);
        }

        let result = domain.call.populate_physmap(domain.domid, count, pfn_shift, 0, &extents).await?;
        if result.len() == 0 {
            return Ok(0);
        }
        *nr_pfns = result.len() << pfn_shift;
        Ok(result.len())
    }
}
