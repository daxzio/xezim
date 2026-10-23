//! §23.6: in a hierarchical name, the segment after a named `begin`/`fork`
//! block is looked up in that block. `U[0].V.a`, where `a` belongs to the
//! generate block `U` and `V` declares nothing, is an error (ivtest
//! `pr1988302b`; the reference simulator reports "Failed to find 'a' in
//! hierarchical name"). The runtime resolver fell back outward and read the
//! generate block's `a`.

use xezim::simulate;

const DESIGN: &str = r#"
module main;
  generate
    genvar i;
    for (i = 0; i < 4; i = i + 2) begin : U
      reg [1:0] a;
      initial begin : V
        reg [1:0] own;
        a = 2'b0;
        own = i;
      end
    end
  endgenerate
  initial #1 $display("H|%0d", REF);
endmodule
"#;

#[test]
fn name_not_declared_in_the_named_block_is_rejected() {
    assert!(simulate(&DESIGN.replace("REF", "U[2].V.a"), 10).is_err());
}

#[test]
fn names_declared_in_the_block_or_its_parent_scope_are_accepted() {
    assert!(simulate(&DESIGN.replace("REF", "U[2].V.own"), 10).is_ok());
    let sim = simulate(&DESIGN.replace("REF", "U[2].a"), 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "H|0"));
}

/// §7.2.2: `resp.exception.valid` is a member select. A `begin : exception`
/// in another module does not make `exception` that block. Verilator 5.050
/// prints PASS. xezim 0.11.2 (5cd31ee5) rejected it as "not declared in
/// block 'exception'" (aionhw/xezim#527).
#[test]
fn struct_member_is_not_a_same_named_block() {
    let src = r#"
typedef struct packed {
  logic       valid;
  logic [5:0] cause;
} exc_t;
typedef struct packed {
  logic [31:0] result;
  exc_t        exception;
} resp_t;
module store;
  initial begin : exception
  end
endmodule
module drv (output resp_t ara_resp_o);
  store i_store ();
  initial begin
    ara_resp_o = '0;
    ara_resp_o.exception.valid = 1'b1;
    ara_resp_o.exception.cause = 6'd2;
  end
endmodule
module t;
  resp_t ara_resp_i;
  resp_t from_drv;
  store i_store ();
  drv i_drv (from_drv);
  initial begin
    ara_resp_i.exception.valid = 1'b1;
    ara_resp_i.exception.cause = 6'd2;
    #1;
    if (ara_resp_i.exception.valid === 1'b1
        && ara_resp_i.exception.cause === 6'd2
        && from_drv.exception.valid === 1'b1
        && from_drv.exception.cause === 6'd2)
      $display("PASS");
    else
      $display("FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("struct select is not a block reference");
    assert!(
        sim.output.iter().any(|o| o.message == "PASS"),
        "output: {:?}",
        sim.output.iter().map(|o| &o.message).collect::<Vec<_>>()
    );
}
