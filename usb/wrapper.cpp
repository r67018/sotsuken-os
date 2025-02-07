#include <cstdint>

#include "usb/xhci/xhci.hpp"
#include "usb/classdriver/mouse.hpp"

char xhc_buf[sizeof(usb::xhci::Controller)];
usb::xhci::Controller* _xhc;

extern "C" {
    typedef struct {
        usb::xhci::Controller impl;
    } UsbXhciControllerImpl;

    UsbXhciControllerImpl* UsbXhciController_UsbXhciController(uint64_t mmio_base) {
        _xhc = new(xhc_buf) usb::xhci::Controller(mmio_base);
        return (UsbXhciControllerImpl*)_xhc;
    }

    void UsbXhciController_UsbXhciController_Destructor(UsbXhciControllerImpl* xhc) {
        delete xhc;
    }

    void UsbXhciController_Initialize(UsbXhciControllerImpl* xhc) {
        xhc->impl.Initialize();
    }

    void UsbXhciController_Run(UsbXhciControllerImpl* xhc) {
        xhc->impl.Run();
    }

    void UsbXhciController_ConfigurePorts(UsbXhciControllerImpl* xhc) {
        for (int i = 1; i <= xhc->impl.MaxPorts(); ++i) {
            auto port = xhc->impl.PortAt(i);
            if (port.IsConnected()) {
                if (auto err = ConfigurePort(xhc->impl, port)) {
                    // TODO: エラー処理
                    continue;
                }
            }
        }
    }

    void UsbXhciController_ProcessEvent(UsbXhciControllerImpl* xhc) {
        while (1) {
            if (auto err = ProcessEvent(xhc->impl)) {
                // TODO: エラー処理
            }
        }
    }

    void SetMouseObserver(usb::HIDMouseDriver::ObserverType observer) {
        usb::HIDMouseDriver::default_observer = observer;
    }
}

// Define to solve the following
// ld.lld: error: undefined symbol: __cxa_pure_virtual
extern "C" void __cxa_pure_virtual() {
    while (1) __asm__("hlt");
}

// libcxx_support.cpp depends on printk function
int printk(const char* format, ...) {
    // noop
}
