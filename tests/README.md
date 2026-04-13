# Rule tests

Each rule has a fixture directory under `tests/fixtures/<rule_name>/`.

Add a `*.c` file to test a rule. Run `BLESS=1 cargo test <rule_name>` to generate the expected `*.stderr`. Optionally add a `*.fixed.c` to also verify the auto-fix output.

Re-run `BLESS=1` whenever expected output legitimately changes.
