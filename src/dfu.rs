pub use usbd_dfu::DFUClass;

use usbd_dfu::DFUMemIO;

pub struct QspiDfu {
    buffer: [u8; Self::TRANSFER_SIZE as usize],
    qspi: crate::external_flash::ExternalFlash<crate::external_flash::Indirect>,
}

impl QspiDfu {
    pub fn new(
        qspi: crate::external_flash::ExternalFlash<crate::external_flash::Indirect>,
    ) -> Self {
        QspiDfu {
            buffer: [0; Self::TRANSFER_SIZE as usize],
            qspi,
        }
    }
}

impl DFUMemIO for QspiDfu {
    const INITIAL_ADDRESS_POINTER: u32 = crate::external_flash::FLASH_START;

    const MEM_INFO_STRING: &'static str =
        "@ExternalFlash/0x90000000/08*004Kg,01*032Kg,63*064Kg,64*064Kg";

    const HAS_DOWNLOAD: bool = true;

    const HAS_UPLOAD: bool = true;

    const MANIFESTATION_TOLERANT: bool = true;

    const PROGRAM_TIME_MS: u32 = 5;

    const ERASE_TIME_MS: u32 = 60;

    const FULL_ERASE_TIME_MS: u32 = 30000;

    const MANIFESTATION_TIME_MS: u32 = 1;

    const DETACH_TIMEOUT: u16 = 250;

    const TRANSFER_SIZE: u16 = 128;

    fn store_write_buffer(&mut self, src: &[u8]) -> Result<(), ()> {
        self.buffer[..src.len()].copy_from_slice(src);
        Ok(())
    }

    fn read(&mut self, address: u32, length: usize) -> Result<&[u8], usbd_dfu::DFUMemError> {
        self.qspi.read_bytes(address, &mut self.buffer[0..length]);
        Ok(&self.buffer[0..length])
    }

    fn program(&mut self, address: u32, length: usize) -> Result<(), usbd_dfu::DFUMemError> {
        let slice = &self.buffer.as_slice()[0..length];

        self.qspi.program_page(address, slice);

        Ok(())
    }

    fn erase(&mut self, address: u32) -> Result<(), usbd_dfu::DFUMemError> {
        self.qspi.block_erase_4k(address);

        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), usbd_dfu::DFUMemError> {
        Ok(())
    }

    fn manifestation(&mut self) -> Result<(), usbd_dfu::DFUManifestationError> {
        Ok(())
    }
}
