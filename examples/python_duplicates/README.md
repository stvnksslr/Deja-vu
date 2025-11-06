# Python Duplicates Example

This directory contains intentional code duplicates for testing Deja-vu.

## Files

### calculator.py
- **Type-1 clones**: `log_operation_v1` and `log_operation_v2` are exact duplicates
- **Type-2 clones**: `validate_input_a` and `validate_input_b` have renamed variables
- Similar patterns in arithmetic functions (add, subtract, multiply, etc.)

### utils.py
- **Type-1 clones**:
  - `format_user_info_v1` and `format_user_info_v2` are exact duplicates
  - Methods in `DataProcessorA` and `DataProcessorB` are duplicated
- **Type-2 clones**:
  - `process_list_method1` and `process_list_method2` with renamed variables
  - `filter_positive_numbers` and `filter_negative_numbers` with slight logic changes

## Testing

Run Deja-vu on this directory:

```bash
# From project root
cargo run --bin deja check examples/python_duplicates/ --verbose

# Or with the CLI
deja check examples/python_duplicates/ -v
```

Expected results:
- Multiple clone groups should be detected
- Both Type-1 and Type-2 clones should be identified
- Different similarity scores based on clone type
