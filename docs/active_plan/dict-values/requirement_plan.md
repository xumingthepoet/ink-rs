# Typed Dict Values Requirement Plan

## Goal

Add first-class typed dictionary values to ink-rs. A dictionary maps keys of one
declared key type to values of one declared value type. The first supported key
types are `string` and `int`.

## Source Syntax

- Dict type names use `Dict<K, V>`.
- `K` must be exactly `string` or `int`.
- `V` may be any maintained value type that can already be stored in variables,
  constants, function parameters, function returns, structs, or arrays.
- Dict literals reuse object literal braces:
  - `{"left": 1, "right": 2}` for `Dict<string, int>`
  - `{1: "left", 2: "right"}` for `Dict<int, string>`
- Existing struct literals keep identifier field keys such as `{ hp: 10 }`.
- Empty `{}` is valid only when the expected type is known, including
  `Dict<K, V>`.

## Semantics

- A typed declaration without an initializer defaults to an empty Dict.
- `dict[key]` reads a Dict value and returns `V`.
- `dict[key] = value` inserts or replaces an entry and stores the updated Dict
  back into the lvalue chain.
- Array indexing remains unchanged and still requires an `int` index.
- Dict indexing requires the declared key type.
- Reading a missing Dict key is a runtime error.
- Dict values compare by value with `==` and `!=`, recursively comparing nested
  values.

## Runtime And JSON

- Runtime owns execution values and gains a Dict value representation.
- The format crate remains the typed compiled-story JSON owner and gains a
  reversible Dict wire value.
- Dict JSON must preserve key type. String key `"1"` and int key `1` must not
  collapse into the same JSON object field.
- Existing array, struct/object, primitive, divert target, and interface value
  encodings remain compatible.
- Save-state JSON uses the same runtime value encoding path as other dynamic
  values.

## V1 Non-Goals

- No Dict iteration syntax.
- No `LEN(dict)`.
- No remove, contains, keys, or values builtin.
- No non-`string`/`int` keys.
- No implicit conversion between string keys and int keys.

