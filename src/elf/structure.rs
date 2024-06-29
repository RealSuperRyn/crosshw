#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawELFHeaderBegin {
    //If no simpler terminology is found, the spec's terminology will be used.
    elfmagic: [u8; 4],
    elfclass: u8,
    elfendian: u8,
    elfheaderversion: u8,
    elfabikind: u8,
    elfabiversion: u8,
    uselesspadding: [u8; 7],
    elftype: u16,
    elfinstructionsetarch: u16,
    elfversion: u32, //32 and 64 bit headers diverge after this
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawELF32HeaderMiddle {
    //We use a middlepoint because after the section header pointer, the 32 and 64 bit headers stop diverging.
    entrypointptr: u32,
    programheaderptr: u32,
    sectionheaderptr: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawELF64HeaderMiddle {
    entrypointptr: u64,
    programheaderptr: u64,
    sectionheaderptr: u64,
}
#[derive(Copy, Clone)]
pub enum RawELFHeaderMiddle {
    _32bit(RawELF32HeaderMiddle),
    _64bit(RawELF64HeaderMiddle),
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawELFHeaderEnd {
    elfflags: u32,
    elfheadersize: u16,
    programheadertableentrysize: u16, //Don't mind the verboseness
    programheadertableentrycount: u16,
    sectionheadertableentrysize: u16,
    sectionheadertableentrycount: u16,
    sectionheadernameindex: u16,
}
#[derive(Copy, Clone)]
pub struct RawELFHeader {
    begin: RawELFHeaderBegin,
    middle: RawELFHeaderMiddle,
    end: RawELFHeaderEnd,
}
impl RawELFHeader {
    pub unsafe fn from_ptr(pointer: *mut u8) -> RawELFHeader {
        let begin = unsafe { *(pointer as *mut RawELFHeaderBegin) };
        let middle: RawELFHeaderMiddle = unsafe {
            if begin.elfclass == 1 {
                RawELFHeaderMiddle::_32bit(*(pointer.offset(24) as *mut RawELF32HeaderMiddle))
            } else {
                RawELFHeaderMiddle::_64bit(*(pointer.offset(24) as *mut RawELF64HeaderMiddle))
            }
        }; //24 offset is the size of RawELFHeaderInitial
        let offsetamount = match middle {
            RawELFHeaderMiddle::_32bit(_) => 36, //24+12, for sizeof(RawELFHeaderBegin) + sizeof(RawELF32HeaderMiddle)
            RawELFHeaderMiddle::_64bit(_) => 48, //24+24, for sizeof(RawELFHeaderBegin) + sizeof(RawELF64HeaderMiddle)
        };
        let end = unsafe { *(pointer.offset(offsetamount) as *mut RawELFHeaderEnd) };
        RawELFHeader {
            begin: begin,
            middle: middle,
            end: end,
        }
    }
}
#[repr(u16)]
#[derive(Debug)]
pub enum EXEtype {
    Unspecified = 0,
    Relocatable = 1,
    Executable = 2,
    Shared = 3,
    Core = 4,
    Unknown(u16),
}
impl EXEtype {
    pub fn from_u16(num: u16) -> EXEtype {
        match num {
            0 => EXEtype::Unspecified,
            1 => EXEtype::Relocatable,
            2 => EXEtype::Executable,
            3 => EXEtype::Shared,
            4 => EXEtype::Core,
            _ => EXEtype::Unknown(num),
        }
    }
}
#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug)]
pub enum Architecture {
    Unknown(u16),
    Sparc = 0x02,
    x86 = 0x03,
    MIPS = 0x08,
    PowerPC = 0x14,
    ARM = 0x28,
    SuperH = 0x2A,
    Itanium64 = 0x32,
    x86_64 = 0x3E,
    AArch64 = 0xB7,
    RISC_V = 0xF3,
}

impl Architecture {
    pub fn from_u16(num: u16) -> Architecture {
        match num {
            0x02 => Architecture::Sparc,
            0x03 => Architecture::x86,
            0x08 => Architecture::MIPS,
            0x14 => Architecture::PowerPC,
            0x28 => Architecture::ARM,
            0x2A => Architecture::SuperH,
            0x32 => Architecture::Itanium64,
            0x3E => Architecture::x86_64,
            0xB7 => Architecture::AArch64,
            0xF3 => Architecture::RISC_V,
            _ => Architecture::Unknown(num),
        }
    }
}
#[derive(Debug)]
pub enum ELFtype {
    _32bit,
    _64bit,
}
#[derive(Debug)]
pub enum Endian {
    LittleEndian,
    BigEndian,
}
#[derive(Debug)]
pub struct ELFInfo {
    pub elf_type: ELFtype,
    pub elf_endian: Endian,
    pub elf_header_version: u8,
    pub abi_kind: u8,
    pub abi_version: u8,
    pub exe_type: EXEtype,
    pub architecture: Architecture,
    pub elf_version: u32,
    pub flags: u32,
    pub elf_header_size: u16,
    pub program_info: ProgramInfo,
    pub section_info: SectionInfo,
    pub elf_pointer: *mut u8,
}
#[derive(Debug)]
pub struct ProgramInfo {
    pub entry_offset: u64,
    pub program_header_offset: u64,
    pub program_header_entry_size: u16,
    pub program_header_entries: u16,
}
#[derive(Debug)]
pub struct SectionInfo {
    pub section_header_offset: u64,
    pub section_header_entry_size: u16,
    pub section_header_entries: u16,
    pub section_string_table_index: u16,
}

impl ELFInfo {
    pub unsafe fn from_ptr(pointer: *mut u8) -> Option<ELFInfo> {
        let raw = unsafe { RawELFHeader::from_ptr(pointer) };
        //We parse the raw header now.
        if raw.begin.elfmagic != [0x7F, 0x45, 0x4c, 0x46] {
            return None;
        }
        let elf_type: ELFtype = match raw.begin.elfclass {
            1 => ELFtype::_32bit,
            2 => ELFtype::_64bit,
            _ => return None,
        };
        let elf_endian: Endian = match raw.begin.elfendian {
            1 => Endian::LittleEndian,
            2 => Endian::BigEndian,
            _ => return None,
        };
        let exe_type: EXEtype = EXEtype::from_u16(raw.begin.elftype);
        let architecture: Architecture = Architecture::from_u16(raw.begin.elfinstructionsetarch);
        let program_info: ProgramInfo = match raw.middle {
            RawELFHeaderMiddle::_32bit(mid) => ProgramInfo {
                entry_offset: mid.entrypointptr as u64,
                program_header_offset: mid.programheaderptr as u64,
                program_header_entry_size: raw.end.programheadertableentrysize,
                program_header_entries: raw.end.programheadertableentrycount,
            },
            RawELFHeaderMiddle::_64bit(mid) => ProgramInfo {
                entry_offset: mid.entrypointptr,
                program_header_offset: mid.programheaderptr,
                program_header_entry_size: raw.end.programheadertableentrysize,
                program_header_entries: raw.end.programheadertableentrycount,
            },
        };
        let section_info: SectionInfo = match raw.middle {
            RawELFHeaderMiddle::_32bit(mid) => SectionInfo {
                section_header_offset: mid.sectionheaderptr as u64,
                section_header_entry_size: raw.end.sectionheadertableentrysize,
                section_header_entries: raw.end.sectionheadertableentrycount,
                section_string_table_index: raw.end.sectionheadernameindex,
            },
            RawELFHeaderMiddle::_64bit(mid) => SectionInfo {
                section_header_offset: mid.sectionheaderptr,
                section_header_entry_size: raw.end.sectionheadertableentrysize,
                section_header_entries: raw.end.sectionheadertableentrycount,
                section_string_table_index: raw.end.sectionheadernameindex,
            },
        };
        Some(ELFInfo {
            elf_type: elf_type,
            elf_endian: elf_endian,
            elf_header_version: raw.begin.elfheaderversion,
            abi_kind: raw.begin.elfabikind,
            abi_version: raw.begin.elfabiversion,
            exe_type: exe_type,
            architecture: architecture,
            elf_version: raw.begin.elfversion,
            flags: raw.end.elfflags,
            elf_header_size: raw.end.elfheadersize,
            program_info: program_info,
            section_info: section_info,
            elf_pointer: pointer,
        })
    }
}
