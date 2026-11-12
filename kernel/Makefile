KDIR ?= $(HOME)/kernel/linux-7.3.4

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) LLVM=1 modules

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) LLVM=1 clean

.PHONY: all clean