# Distillation Shortcut CAPE-OPEN Unit Operation

This is an example unit operation implementation featuring 
a shortcut distillation unit operation using the 
Fenske-Underwood-Gilliland-Kirkbride approach. 

The unit operation itself is implemented in-process in a
shared libary.

The editor is implemented using Tauri v2, and runs
out-of-process from the CAPE-OPEN compliant flowsheeting environment, 
as Tauri (or rather tao) does not like running in a foreign thread, 
which is not something the CAPE-OPEN unit operation has control over.

In addition when running in-process, tao tends to leave input handling
hooks after the main dialog terminates, and the easiest way to get rid of those is 
to terminate the process after editing is done.

The two projects are:

* distillation_shortcut_unit - the CAPE-OPEN Unit Operation implementation
* distillation_shortcut_unit_editor - the Tauri v2 based editor dialog

These are combined into a single workspace.

## build instructions

### Debug

```
cargo build
cobiaRegister target/debug/distillation_shortcut_unit.dll
```

On linux, replace ```distillation_shortcut_unit.dll``` by ```libdistillation_shortcut_unit.so```.

### Release

```
cargo build --release
cobiaRegister target/release/distillation_shortcut_unit.dll
```

On linux, replace ```distillation_shortcut_unit.dll``` by ```libdistillation_shortcut_unit.so```.

```

### Documentation

```
cargo doc --no-deps --document-private-items --open
```
