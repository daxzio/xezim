//! §7.2.2 / §6.20.2 — `array[i].field.subfield` on a localparam array of
//! packed structs. One-level `array[i].field` is that field. The extra
//! select used to miss the layout (keyed on the root parameter) and become
//! a 32-bit 0. Verilator 5.052 is the reference; Icarus rejects unpacked
//! array parameters.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("sim")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn nested_field_of_array_param() {
    let got = lines(
        r#"
typedef struct packed {
  logic       accept;
  logic [1:0] gpr;
} inner_t;
typedef struct packed {
  logic [31:0] instr;
  inner_t      rsp;
} outer_t;
module t;
  localparam inner_t INNERS[1] = '{'{accept: 1'b1, gpr: 2'b01}};
  localparam outer_t OUTERS[1] = '{'{instr: 32'h0400702b, rsp: '{accept: 1'b1, gpr: 2'b01}}};
  inner_t tmp;
  initial begin
    tmp = OUTERS[0].rsp;
    $display("inner_arr=%b bits=%0d", INNERS[0].accept, $bits(INNERS[0].accept));
    $display("chain=%b bits=%0d", OUTERS[0].rsp.accept, $bits(OUTERS[0].rsp.accept));
    $display("whole=%h", OUTERS[0].rsp);
    $display("tmp=%b bits=%0d", tmp.accept, $bits(tmp.accept));
    $finish;
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        vec![
            "inner_arr=1 bits=1",
            "chain=1 bits=1",
            "whole=5",
            "tmp=1 bits=1",
        ]
    );
}
