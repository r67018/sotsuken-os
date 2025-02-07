TARGET = libusb.a
OBJS = wrapper.o \
       logger.o \
       libcxx_support.o \
       newlib_support.o \
       pci.o \
	   timer.o \
       usb/memory.o \
       usb/device.o \
       usb/xhci/ring.o \
       usb/xhci/trb.o \
       usb/xhci/xhci.o \
       usb/xhci/port.o \
       usb/xhci/device.o \
       usb/xhci/devmgr.o  \
       usb/xhci/registers.o \
       usb/classdriver/base.o \
       usb/classdriver/hid.o \
       usb/classdriver/keyboard.o \
       usb/classdriver/mouse.o
DEPENDS = $(join $(dir $(OBJS)),$(addprefix .,$(notdir $(OBJS:.o=.d))))

CPPFLAGS += -I.
CFLAGS   += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone
CXXFLAGS += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone \
            -fno-exceptions -fno-rtti -std=c++17
MIKANDIR = ../mikanos/kernel

.PHONY: all
all:
	$(MAKE) setup
	$(MAKE) $(TARGET)

.PHONY: clean
clean:
	find . $(MIKANDIR) -name "*.o" | xargs rm -rf
	find . $(MIKANDIR) -name "*.d" | xargs rm -rf
	find . -type l | xargs rm
	rm libusb.a

.PHONY: setup
setup:
	find ../mikanos/kernel \( -type f -not -name "logger.cpp" \( -name "*.c" -o -name "*.cpp" -o -name "*.h" -o -name "*.hpp" \) \) -o -type d | xargs -I@ ln -sf @ .

$(TARGET): $(OBJS) Makefile
	ar rcs $@ $(OBJS)

%.o: %.cpp Makefile
	clang++ $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

.%.d: %.cpp
	clang++ $(CPPFLAGS) $(CXXFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.cpp=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

%.o: %.c Makefile
	clang $(CPPFLAGS) $(CFLAGS) -c $< -o $@

.%.d: %.c
	clang $(CPPFLAGS) $(CFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.c=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

.PHONY: depends
depends:
	$(MAKE) $(DEPENDS)

-include $(DEPENDS)

