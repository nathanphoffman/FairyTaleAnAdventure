# Builds the C part of the port on its own, to check it compiles and links.
# The real build goes through cargo (build.rs) once the Rust platform layer exists.
CC      = gcc
CFLAGS  = -std=gnu89 -fcommon -g -O0 -I. -Istubs
OBJDIR  = ../build/obj
OBJS    = $(addprefix $(OBJDIR)/,fmain.o fmain2.o iffsubs.o hdrive.o platform_stubs.o asm_stubs.o)

all: ../build/fta_c

../build/fta_c: $(OBJS)
	$(CC) -o $@ $(OBJS)

$(OBJDIR)/%.o: %.c ftale.h amiga.h asm_protos.h protos.h | $(OBJDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJDIR):
	mkdir -p $(OBJDIR)

protos.h: fmain.c fmain2.c iffsubs.c hdrive.c
	../tools/genprotos.sh

clean:
	rm -rf ../build
