#[cfg(test)]
pub mod cartridge_tests {
    use crate::hw::cartridge::{Cartridge, CartridgeError, ScreenMirroring};
    
    fn create_valid_ines_header() -> Vec<u8> {
        vec![
            0x4E, 0x45, 0x53, 0x1A,
            0x01,
            0x01,
            0x00,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn create_test_cartridge_data(header: Vec<u8>, prg_size: usize, chr_size: usize) -> Vec<u8> {
        let mut data = header;


        data.extend(vec![0x42; prg_size]);


        data.extend(vec![0x33; chr_size]);

        data
    }

    #[test]
    fn test_valid_cartridge_creation() {
        let header = create_valid_ines_header();
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();

        assert_eq!(cartridge.prg_rom.len(), 16384);
        assert_eq!(cartridge.chr_rom.len(), 8192);
        assert_eq!(cartridge.mapper, 0);
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::Horizontal);


        assert!(cartridge.prg_rom.iter().all(|&b| b == 0x42));
        assert!(cartridge.chr_rom.iter().all(|&b| b == 0x33));
    }

    #[test]
    fn test_invalid_header() {
        let mut data = create_valid_ines_header();
        data[0] = 0x4F;
        let data = create_test_cartridge_data(data, 16384, 8192);

        let result = Cartridge::new(data);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let cartridge_error = error.downcast::<CartridgeError>().unwrap();

        match cartridge_error {
            CartridgeError::InvalidHeader { expected, found } => {
                assert_eq!(expected, vec![0x4E, 0x45, 0x53, 0x1A]);
                assert_eq!(found, vec![0x4F, 0x45, 0x53, 0x1A]);
            }
            _ => panic!("Expected InvalidHeader error"),
        }
    }

    #[test]
    fn test_nes2_format_not_supported() {
        let mut header = create_valid_ines_header();
        header[7] = 0b0000_1000;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let result = Cartridge::new(data);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let cartridge_error = error.downcast::<CartridgeError>().unwrap();

        match cartridge_error {
            CartridgeError::UnsupportedINESVersion => {}
            _ => panic!("Expected UnsupportedINESVersion error"),
        }
    }

    #[test]
    fn test_vertical_mirroring() {
        let mut header = create_valid_ines_header();
        header[6] = 0b0000_0001;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::Vertical);
    }

    #[test]
    fn test_horizontal_mirroring() {
        let mut header = create_valid_ines_header();
        header[6] = 0b0000_0000;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::Horizontal);
    }

    #[test]
    fn test_four_screen_mirroring() {
        let mut header = create_valid_ines_header();
        header[6] = 0b0000_1000;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::FourScreen);
    }

    #[test]
    fn test_four_screen_overrides_other_mirroring() {
        let mut header = create_valid_ines_header();
        header[6] = 0b0000_1001;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::FourScreen);
    }

    #[test]
    fn test_mapper_calculation() {
        let mut header = create_valid_ines_header();
        header[6] = 0b1111_0000;
        header[7] = 0b1111_0000;
        let data = create_test_cartridge_data(header, 16384, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.mapper, 0b1111_1111);
    }

    #[test]
    fn test_different_prg_rom_sizes() {
        let test_cases = vec![
            (1, 16384),
            (2, 32768),
            (4, 65536),
        ];

        for (page_count, expected_size) in test_cases {
            let mut header = create_valid_ines_header();
            header[4] = page_count;
            let data = create_test_cartridge_data(header, expected_size, 8192);

            let cartridge = Cartridge::new(data).unwrap();
            assert_eq!(cartridge.prg_rom.len(), expected_size);
        }
    }

    #[test]
    fn test_different_chr_rom_sizes() {
        let test_cases = vec![
            (0, 0),
            (1, 8192),
            (2, 16384),
        ];

        for (page_count, expected_size) in test_cases {
            let mut header = create_valid_ines_header();
            header[5] = page_count;
            let data = create_test_cartridge_data(header, 16384, expected_size);

            let cartridge = Cartridge::new(data).unwrap();
            assert_eq!(cartridge.chr_rom.len(), expected_size);
        }
    }

    #[test]
    fn test_trainer_present() {
        let mut header = create_valid_ines_header();
        header[6] = 0b0000_0100;

        let mut data = header;

        data.extend(vec![0x11; 512]);

        data.extend(vec![0x42; 16384]);

        data.extend(vec![0x33; 8192]);

        let cartridge = Cartridge::new(data).unwrap();

        assert_eq!(cartridge.prg_rom.len(), 16384);
        assert_eq!(cartridge.chr_rom.len(), 8192);
        assert!(cartridge.prg_rom.iter().all(|&b| b == 0x42));
        assert!(cartridge.chr_rom.iter().all(|&b| b == 0x33));
    }

    #[test]
    fn test_zero_prg_rom_size() {
        let mut header = create_valid_ines_header();
        header[4] = 0;
        let data = create_test_cartridge_data(header, 0, 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.prg_rom.len(), 0);
        assert_eq!(cartridge.chr_rom.len(), 8192);
    }

    #[test]
    fn test_large_rom_sizes() {
        let mut header = create_valid_ines_header();
        header[4] = 16;
        header[5] = 8;
        let data = create_test_cartridge_data(header, 16 * 16384, 8 * 8192);

        let cartridge = Cartridge::new(data).unwrap();
        assert_eq!(cartridge.prg_rom.len(), 262144);
        assert_eq!(cartridge.chr_rom.len(), 65536);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Cartridge::INES_TAG, [0x4E, 0x45, 0x53, 0x1A]);
        assert_eq!(Cartridge::PRG_ROM_PAGE_SIZE, 16384);
        assert_eq!(Cartridge::CHR_ROM_PAGE_SIZE, 8192);
        assert_eq!(Cartridge::HEADER_SIZE, 16);
        assert_eq!(Cartridge::TRAINER_SIZE, 512);
    }

    #[test]
    fn test_error_display() {
        let invalid_header_error = CartridgeError::InvalidHeader {
            expected: vec![0x4E, 0x45, 0x53, 0x1A],
            found: vec![0x4F, 0x45, 0x53, 0x1A],
        };

        let error_msg = format!("{}", invalid_header_error);
        assert!(error_msg.contains("Invalid INES header"));
        assert!(error_msg.contains("expected"));
        assert!(error_msg.contains("got"));

        let version_error = CartridgeError::UnsupportedINESVersion;
        assert_eq!(format!("{}", version_error), "NES2.0 format is not supported");

        let mirroring_error = CartridgeError::IllegalScreenMirroring;
        assert_eq!(format!("{}", mirroring_error), "Illegal screen mirroring found");
    }

    #[test]
    fn test_screen_mirroring_debug() {
        assert_eq!(format!("{:?}", ScreenMirroring::Horizontal), "Horizontal");
        assert_eq!(format!("{:?}", ScreenMirroring::Vertical), "Vertical");
        assert_eq!(format!("{:?}", ScreenMirroring::FourScreen), "FourScreen");
    }

    #[test]
    fn test_screen_mirroring_equality() {
        assert_eq!(ScreenMirroring::Horizontal, ScreenMirroring::Horizontal);
        assert_ne!(ScreenMirroring::Horizontal, ScreenMirroring::Vertical);
        assert_ne!(ScreenMirroring::Vertical, ScreenMirroring::FourScreen);
    }

    #[test]
    fn test_minimal_valid_rom() {
        let data = vec![
            0x4E, 0x45, 0x53, 0x1A,
            0x01,
            0x00,
            0x00,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut complete_data = data;
        complete_data.extend(vec![0x55; 16384]);

        let cartridge = Cartridge::new(complete_data).unwrap();
        assert_eq!(cartridge.prg_rom.len(), 16384);
        assert_eq!(cartridge.chr_rom.len(), 0);
        assert_eq!(cartridge.mapper, 0);
        assert_eq!(cartridge.screen_mirroring, ScreenMirroring::Horizontal);
    }
}