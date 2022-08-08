use crate::error::Error;
use core::slice;
use acpi::{
    AmlTable,
    AcpiHandler,
    PhysicalMapping,
    AcpiTables,
    fadt::Fadt,
    bgrt::Bgrt,
    madt::Madt,
    hpet::HpetTable,
    mcfg::Mcfg
};
use aml::{
    DebugVerbosity,
    AmlContext,
    Handler as AmlHandler,
    AmlValue,
    AmlName
};
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;

pub(crate) static ACPI_INFO: OnceCell<AcpiInfo> = OnceCell::uninit();

pub(crate) struct AcpiInfo {
    pub pm1a_control_block: Option<u64>,
    pub aml_context: Option<AmlContext>
}

#[derive(Clone)]
pub struct RicosAcpiHandler {
    physical_memory_offset: u64
}

impl AcpiHandler for RicosAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize
    ) -> PhysicalMapping<Self, T> {
        let virtual_start = self.physical_memory_offset + physical_address as u64;
        PhysicalMapping::new(
            physical_address,
            core::ptr::NonNull::new_unchecked(virtual_start as *mut u8 as *mut T),
            size,
            size,
            Self {
                physical_memory_offset: self.physical_memory_offset
            }
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        //do nothing, whole physical memory is mapped
    }
}

struct RicosAmlHandler;

#[allow(unused_variables)]
impl AmlHandler for RicosAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        unimplemented!()
    }

    fn read_u16(&self, address: usize) -> u16 {
        unimplemented!()
    }

    fn read_u32(&self, address: usize) -> u32 {
        unimplemented!()
    }

    fn read_u64(&self, address: usize) -> u64 {
        unimplemented!()
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        unimplemented!()
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        unimplemented!()
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        unimplemented!()
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        unimplemented!()
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unimplemented!()
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unimplemented!()
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unimplemented!()
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unimplemented!()
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unimplemented!()
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        unimplemented!()
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        unimplemented!()
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        unimplemented!()
    }

    fn write_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u8) {
        unimplemented!()
    }

    fn write_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u16) {
        unimplemented!()
    }

    fn write_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u32) {
        unimplemented!()
    }
}

#[allow(unused_variables)]
pub fn init_acpi_info(
    physical_memory_offset: u64,
    verbose: bool
) -> Result<(), Error> {
    let acpi_tables =  unsafe { AcpiTables::search_for_rsdp_bios(RicosAcpiHandler{physical_memory_offset}) }?;

    if verbose && log::log_enabled!(log::Level::Trace) {
        log::trace!("ACPI revision: {}", acpi_tables.revision);

        let platform_info = acpi_tables.platform_info()?;
        log::trace!("Platform info");
        log::trace!("power_profile={:?}", platform_info.power_profile);
        log::trace!("interrupt_model={:?}", platform_info.interrupt_model);
        if let Some(pm_timer) = platform_info.pm_timer {
            log::trace!("pm_timer: base={:?}, supports_32bit={:?}", pm_timer.base, pm_timer.supports_32bit);
        }
        if let Some(processor_info) = platform_info.processor_info {
            log::trace!("boot_processor={:?}", processor_info.boot_processor);
            for application_processor in processor_info.application_processors {
                log::trace!("application_processor={:?}", application_processor);
            }
        }
    }

    let fadt = unsafe { acpi_tables.get_sdt::<Fadt>(acpi::sdt::Signature::FADT)? } ;
    let pm1a_control_block = if let Some(fadt) = fadt {
        log::info!("fadt found");
        let pm1a_control_block = fadt.pm1a_control_block()?;
        log::info!("pm1a_control_block=0x{:04x}", pm1a_control_block.address);
        Some(pm1a_control_block.address)
    } else {
        None
    };

    if verbose && log::log_enabled!(log::Level::Trace) {
        let bgrt = unsafe { acpi_tables.get_sdt::<Bgrt>(acpi::sdt::Signature::BGRT)? } ;
        if let Some(bgrt) = bgrt {
            log::trace!("bgrt found");
        }

        let hpet = unsafe { acpi_tables.get_sdt::<HpetTable>(acpi::sdt::Signature::HPET)? } ;
        if let Some(hpet) = hpet {
            log::trace!("hpet found");
        }

        let madt = unsafe { acpi_tables.get_sdt::<Madt>(acpi::sdt::Signature::MADT)? } ;
        if let Some(madt) = madt {
            log::trace!("madt found");
        }

        let mcfg = unsafe { acpi_tables.get_sdt::<Mcfg>(acpi::sdt::Signature::MCFG)? } ;
        if let Some(mcfg) = mcfg {
            log::trace!("mcfg found");
        }
    }

    let mut aml_context = AmlContext::new(Box::new(RicosAmlHandler), DebugVerbosity::None);
    if let Some(dsdt) = acpi_tables.dsdt {
        log::info!("Parsing dsdt...");
        parse_aml_table(RicosAcpiHandler{physical_memory_offset}, &mut aml_context, &dsdt);
    }
    log::trace!("{} ssdts found.", acpi_tables.ssdts.len());
    for ssdt in acpi_tables.ssdts {
        log::info!("Parsing ssdt...");
        parse_aml_table(RicosAcpiHandler{physical_memory_offset}, &mut aml_context, &ssdt);
    }
    // aml_context.initialize_objects()?;
    if verbose {
        log::trace!("{:?}", aml_context.namespace);
    }

    ACPI_INFO.init_once(|| AcpiInfo {
        pm1a_control_block,
        aml_context: Some(aml_context)
    });

    Ok(())
}

fn parse_aml_table(handler: RicosAcpiHandler, context: &mut AmlContext, aml_table: &AmlTable) {
    let physical_region = unsafe { handler.map_physical_region::<u8>(aml_table.address, aml_table.length as usize) };
    let stream = unsafe { slice::from_raw_parts(physical_region.virtual_start().as_ptr(), aml_table.length as usize) };

    if let Err(err) = context.parse_table(stream) {
        log::error!("Error parsing aml table: {:?}", err);
    }
}

pub fn get_shutdown_info() -> Option<(u16,u16)> {
    if let Some(acpi_info) = ACPI_INFO.get() {
        if let Some(pm1a_control_block) = acpi_info.pm1a_control_block {
            if let Some(aml_context) = &acpi_info.aml_context {
                let path = AmlName::from_str("\\_S5").unwrap();
                if let Ok(AmlValue::Package(val)) = aml_context.namespace.get_by_path(&path) {
                    let slp_typa = &val[0].as_integer(&aml_context).unwrap();
                    let slp_typb = &val[1].as_integer(&aml_context).unwrap();

                    let mut val = 1 << 13;
                    val |= slp_typa;

                    log::info!("shutdown_info: pm1a_control_block=0x{:04x}, slp_typa=0x{:04x}, slp_typb=0x{:04x}, val=0x{:04x}", pm1a_control_block, slp_typa, slp_typb, val);

                    Some((pm1a_control_block as u16, val as u16))
                } else {
                    log::error!("couldn't get \\_S5 from aml_context");
                    None
                }
            } else {
                log::error!("aml_context is not present");
                None
            }
        } else {
            log::error!("pm1a_control_block is not present");
            None
        }
    } else {
        log::error!("acpi_info not initialized");
        None
    }
}
