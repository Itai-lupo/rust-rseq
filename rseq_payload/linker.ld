ENTRY(_start)

SECTIONS {
     . = 0 + SIZEOF_HEADERS;
    .hash          : { *(.hash) } :text
    .gnu.hash      : { *(.gnu.hash) } :text
    .dynsym        : { *(.dynsym) } :text
    .dynstr        : { *(.dynstr) } :text
    .rela.dyn      : { *(.rela.dyn) } :text
    .rela.plt      : { *(.rela.plt) } :text

    .text : ALIGN(32) {
        rseq_start = .;
        *(.text .text.*)
        *(.gnu.linkonce.t.*)

        KEEP(*(.rseq_critical))
      

    } :text

    .text.rseq_commit : {
        KEEP(*(.rseq_commit))
        rseq_commit_end = .;
    } :text

    .rseq_abort : {
        rseq_abort_ip = . + 4;
        KEEP(*(.rseq_abort))
    } :text
    
    .rodata : {
        *(.rodata .rodata.*)
        *(.gnu.linkonce.r.*)
    } :text

    . = ALIGN(4096);
    .dynamic : { *(.dynamic) } :data :dynamic
    .got     : { *(.got .got.plt) } :data
    
    . = ALIGN(0x1000);
    .data : {
        *(.data .data.*)
        *(.gnu.linkonce.d.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

   .tdata : ALIGN(16) {
        _tls_start = .;
        *(.tdata .tdata.*)
    } :data :tls

   .tbss : ALIGN(16) {
        *(.tbss .tbss.*)
        _tls_end = .;
    } :data :tls

   /DISCARD/ : { *(.comment) *(.note.*)   *(.eh_frame) }
}

VERSION {
    VERS_1.0 {
        global:
            rseq_abort_ip;
            rseq_commit_end;
            rseq_start;
        local:
            *; 
        };
}

PHDRS
{
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
    dynamic PT_DYNAMIC;
    tls PT_TLS;
    gnu_stack PT_GNU_STACK FLAGS(6);
}


