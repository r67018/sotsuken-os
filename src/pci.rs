use anyhow::anyhow;
use thiserror::Error;

extern "C" {
    /// IOポート空間にデータを書き込む
    fn IoOut32(address: u16, data: u32);
    /// IOポート空間からデータを読み込む
    fn IoIn32(address: u16) -> u32;
}

/// レジスタのIOポートアドレス
/// 読み取りたいデータのアドレスを指定する
const CONFIG_ADDRESS: u16 = 0x0cf8;
/// レジスタのIOポートアドレス
/// CONFIG_ADDRESSの値を元に、CONFIG_DATAのアドレスに実際のデータが存在する
const CONFIG_DATA: u16 = 0x0cfc;

/// PCIデバイスのクラスコード
#[derive(Debug, Clone, Copy)]
pub struct ClassCode {
    pub base: u8,
    pub sub: u8,
    pub interface: u8,
}

impl ClassCode {
    pub fn match_base(&self, b: u8) -> bool {
        self.base == b
    }

    pub fn match_base_sub(&self, b: u8, s: u8) -> bool {
        self.match_base(b) && self.sub == s
    }

    pub fn match_base_sub_interface(&self, b: u8, s: u8, i: u8) -> bool {
        self.match_base_sub(b, s) && self.interface == i
    }
}

/// PCIデバイスを操作するための基礎データを格納する
#[derive(Debug, Copy, Clone)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: ClassCode,
}

/// CONFIG_ADDRESS用の32ビットアドレスを生成する
fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    // 左ビットシフト
    fn shl(x: u32, bits: u32) -> u32 {
        x << bits
    }

    shl(1, 31)
    | shl(bus as u32, 16)
    | shl(device as u32, 11)
    | shl(function as u32, 8)
    | (reg_addr as u32 & 0xfc)
}

/// CONFIG_ADDRESSにアドレスを書き込む
unsafe fn write_address(address: u32) {
    IoOut32(CONFIG_ADDRESS, address);
}

/// CONFIG_DATAの場所に格納されたデータに値を書き込む
unsafe fn write_data(value: u32) {
    IoOut32(CONFIG_DATA, value)
}

unsafe fn read_data() -> u32 {
    IoIn32(CONFIG_DATA)
}

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    unsafe {
        write_address(make_address(bus, device, function, 0x00));
        (read_data() & 0xffff) as u16
    }
}

unsafe fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    ((read_data() >> 16) & 0xff) as u8
}

pub unsafe fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    write_address(make_address(bus, device, function, 0x08));
    let reg = read_data();
    ClassCode {
        base: (reg >> 24) as u8,
        sub: (reg >> 16) as u8,
        interface: (reg >> 8) as u8,
    }
}

unsafe fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn read_conf_reg(device: &Device, reg_addr: u8) -> u32 {
    unsafe {
        write_address(make_address(device.bus, device.device, device.function, reg_addr));
        read_data()
    }
}

pub fn write_conf_reg(dev: &Device, reg_addr: u8, value: u32) {
    unsafe {
        write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
        write_data(value);
    }
}

/// BAR(Base Address Register)へのアドレスを取得  
/// BAR0からBAR5まであるため、取得したいBARをインデックスで指定する  
/// BARはレジスタだが、MMIO(メモリ空間にマッピングされたIO)なので、アドレス先を読み取ることで取得できる
pub fn read_bar(device: &Device, bar_index: usize) -> anyhow::Result<u64> {
    // アドレスを計算
    fn calc_bar_address(bar_index: usize) -> u8 {
        0x10 + 4 * bar_index as u8
    }

    if bar_index < 0 || 6 <= bar_index {
        return Err(anyhow!(PciError::IndexOutOfRange { min: 0, max: 5, actual: bar_index }));
    }

    let addr = calc_bar_address(bar_index);
    let bar = read_conf_reg(device, addr) as u64;

    // 32ビットアドレスの場合はそのまま返す
    if (bar & 4) == 0 {
        return Ok(bar);
    }

    // 64ビットアドレスの場合、連続した2つのBARを使う
    // 64ビットアドレスで、かつBARのインデックスが5以上ならエラー
    if bar_index >= 5 {
        return Err(anyhow!(PciError::IndexOutOfRange { min: 0, max: 4, actual: bar_index }));
    }

    // 上位32ビットも取得する
    let bar_upper = read_conf_reg(device, addr + 4) as u64;
    Ok(bar_upper << 32 | bar)
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

fn is_invalid_vendor_id(id: u16) -> bool {
    id == 0xffff
}

pub struct PciBus {
    devices: [Device; 32],
    pub num_device: usize,
}

impl PciBus {
    pub fn new() -> Self {
        Self {
            devices: [Device {
                bus: 0,
                device: 0,
                function: 0,
                header_type: 0,
                class_code: ClassCode {
                    base: 0,
                    sub: 0,
                    interface: 0,
                },
            }; 32],
            num_device: 0,
        }
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices[0..self.num_device]
    }

    /// 全てのバスをスキャンする
    pub fn scan_all_bus(&mut self) -> anyhow::Result<()> {
        // ホストブリッジ(バス: 0, デバイス: 0, ファンクション: 0)のヘッダタイプを取得する
        let header_type = unsafe { read_header_type(0, 0, 0) };
        // ホストブリッジが単機能デバイスなら、バスは1つしかない
        if is_single_function_device(header_type)  {
            return self.scan_bus(0);
        }

        // 単機能デバイスじゃないなら、ホストブリッジが複数あるので、各バスを探索する
        for function in 1..8 {
            let vendor_id = read_vendor_id(0, 0, function);
            if is_invalid_vendor_id(vendor_id) {
                continue;
            }
            self.scan_bus(function)?
        }

        Ok(())
    }

    /// 指定のバス番号の各デバイスをスキャンする
    fn scan_bus(&mut self, bus: u8) -> anyhow::Result<()> {
        for device in 0..32 {
            let vendor_id = read_vendor_id(bus, device, 0);
            // ベンダIDが無効な場合はスキップ
            if is_invalid_vendor_id(vendor_id) {
                continue;
            }
            self.scan_device(bus, device)?
        }

        Ok(())
    }

    /// 各デバイス番号のファンクションのスキャンする
    fn scan_device(&mut self, bus: u8, device: u8) -> anyhow::Result<()> {
        self.scan_function(bus, device, 0)?;
        let header_type = unsafe { read_header_type(bus, device, 0) };
        if is_single_function_device(header_type) {
            return Ok(());
        }

        // 各ファンクションをスキャンする
        for function in 1..8 {
            let vendor_id = read_vendor_id(bus, device, function);
            // ベンダIDが無効な場合はスキップ
            if is_invalid_vendor_id(vendor_id) {
                continue;
            }
            self.scan_function(bus, device, function)?
        }

        Ok(())
    }

    /// 指定のファンクションをdevicesに追加する
    /// もしPCI-PCIブリッチなら、セカンダリバスに対しScanBusを実行する
    fn scan_function(&mut self, bus: u8, device: u8, function: u8) -> anyhow::Result<()> {
        let header_type = unsafe { read_header_type(bus, device, function) };
        let class_code = unsafe { read_class_code(bus, device, function) };
        self.add_device(bus, device, function, header_type, class_code)?;

        unsafe {
            // ファンクションがPCI-PCIブリッジの場合はセカンダリバスに繋がったデバイスをスキャン
            if class_code.match_base_sub(0x06, 0x04) {
                let bus_numbers = read_bus_numbers(bus, device, function);
                let secondary_bus = (bus_numbers >> 8) as u8;
                return self.scan_bus(secondary_bus);
            }
        }

        Ok(())
    }

    // 発見したPCIデバイスをdevicesに追加する
    fn add_device(&mut self, bus: u8, device: u8, function: u8, header_type: u8, class_code: ClassCode) -> anyhow::Result<()> {
        // devicesが満杯の場合はエラー
        if self.num_device == self.devices.len() {
            return Err(anyhow!(PciError::DeviceIsFull));
        }

        self.devices[self.num_device] = Device {
            bus,
            device,
            function,
            header_type,
            class_code,
        };
        self.num_device += 1;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PciError {
    #[error("Cannot add a PCI device any more since device array is already full.")]
    DeviceIsFull,
    #[error("Index out of range (min: {min}, max: {max}, actual: {actual})")]
    IndexOutOfRange { min: usize, max: usize, actual: usize },
}
