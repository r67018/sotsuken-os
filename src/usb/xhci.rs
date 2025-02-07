use crate::{pci, printk};

enum ControllerImpl {}

type MouseObserver = extern "C" fn(buttons: u8, displacement_x: i8, displacement_y: i8);

extern "C" {
    fn UsbXhciController_UsbXhciController(mmio_base: u64) -> *mut ControllerImpl;
    fn UsbXhciController_Initialize(controller: *mut ControllerImpl);
    fn UsbXhciController_Run(controller: *mut ControllerImpl);
    fn UsbXhciController_ConfigurePorts(controller: *mut ControllerImpl);
    fn UsbXhciController_ProcessEvent(controller: *mut ControllerImpl);
    fn SetMouseObserver(observer: MouseObserver);
}

pub struct Controller {
    raw: *mut ControllerImpl,
}

impl Controller {
    pub fn new(mmio_base: u64) -> Self {
        unsafe { Controller { raw: UsbXhciController_UsbXhciController(mmio_base) } }
    }

    pub fn initialize(&self) {
        unsafe { UsbXhciController_Initialize(self.raw) }
    }

    pub fn run(&self) {
        unsafe { UsbXhciController_Run(self.raw) }
    }
    
    pub fn configure_ports(&self) {
        unsafe { UsbXhciController_ConfigurePorts(self.raw) }
    }
    
    pub fn process_event(&self) {
        unsafe { UsbXhciController_ProcessEvent(self.raw) }
    }
}

/// USBの制御モードを切り替える
pub fn switch_ehci2xhci(pci: &pci::PciBus, xhc_dev: &pci::Device) {
    let mut intel_ehc_exist = false;
    for i in 0..pci.num_device {
        let device = pci.devices()[i];
        if device.class_code.match_base_sub_interface(0xc, 0x3, 0x20) &&
            (pci::read_vendor_id(device.device, device.device, device.function) == 0x8086) {
            intel_ehc_exist = true;
            break;
        }
    }
    if !intel_ehc_exist {
        return;
    }
    
    let superspeed_ports = pci::read_conf_reg(xhc_dev, 0xdc);
    pci::write_conf_reg(xhc_dev, 0xd8, superspeed_ports);
    let ehci2xhci_ports = pci::read_conf_reg(xhc_dev, 0xd4);
    pci::write_conf_reg(xhc_dev, 0xd0, ehci2xhci_ports);
    printk!("switch_ehci2xhci: SS = {:02x}, xHCI = {:02x}\n", superspeed_ports, ehci2xhci_ports);
}

pub fn set_mouse_observer(observer: MouseObserver) {
    unsafe { SetMouseObserver(observer) }
}
