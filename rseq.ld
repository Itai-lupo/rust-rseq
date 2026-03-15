SECTIONS {
  .rseq_payload : {
    _rseq_start = .;
    *(.rseq_logic)
    _rseq_post_commit = .;
    . = ALIGN(4);
    KEEP(*(.rseq_sig))
    _rseq_abort_ip = .;
    *(.rseq_abort)
  }
}
