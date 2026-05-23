# ink-rs JSON runtime format

When ink is compiled to JSON, it is converted to a low level format for use by the runtime, and is made up of smaller, simpler building blocks. For an overview of the full pipeline, including a description of the runtime itself see the [Architecture documentation](Architecture.md).

## Top level

At the top level of the JSON file are `inkVersion`, `root`, and optional
metadata. `inkVersion` is an integer that denotes the format version, and
`root` is the outer-most Container for the entire story.

```json
{
    "inkVersion": 1,
    "root": <root container>,
    "internalFunctions": {
        "config::read_config": {
            "path": "config.read_config",
            "args": 1,
            "argTypes": ["string"],
            "returnType": "string"
        }
    },
    "interfaces": {
        "IItem": {
            "members": {
                "target": "knot",
                "score": "function"
            },
            "implementations": ["left", "right"]
        }
    }
}
```

The current Rust format version is `2`. The runtime only loads compiled story
JSON whose `inkVersion` exactly matches the current format version; older
compiled story JSON versions are not treated as compatible.

Broadly speaking, the entire format is composed of Containers, and individual sub-elements of the Story, within those Containers.

Explicit source modules do not require a new compiled-story JSON schema. The
root container starts by diverting to the unique `module.main` entry point and
stores reachable module containers as root named content. Source-qualified flow
names such as `items::take` are lowered to dot-separated runtime paths such as
`items.take`. Reachable modules include the `main` module import closure plus
modules that declare `INTERNAL` functions and their import closures.

`internalFunctions` is emitted only when the source declares `INTERNAL`
host-callable ink functions. The object key is the source-qualified host API
name. `path` is the runtime container path, `args` is the argument count,
`argTypes` stores source type names, and `returnType` stores the declared return
type. Runtime loading keeps this metadata separate from the runtime execution
graph and uses it to validate `Story::call_internal`.

`interfaces` is emitted when the source declares interfaces. Each key is an
interface name. `members` maps member names to `"knot"` or `"function"`.
`implementations` lists source module names that explicitly implement the
interface and are present in the compiled story. Unreachable implementation
modules are omitted because their containers are not compiled. Runtime loading
keeps this metadata separate from the runtime execution graph and uses it to
validate dynamic interface dispatch.

## Containers

There is only one type of generalised collection, and this is the **Container** - it's used throughout the engine. In JSON it's represented as an array type.

The root of a story is a container, and as a story is evaluated, the engine steps through the sub-elements of containers.

Although containers primarily behave like arrays, they also have additional attributes, including a way to reference named sub-elements that aren't included in the array itself. To support this behaviour, **the final element of the array is special**. The final element is either `null`, or it's an object (dictionary) that contains named sub-elements, such as nested Containers, and optionally `#n`, which holds the name of the container itself when that name is not redundant due to being a named field of a parent container.

Examples:

* `[5, 6, null]` - A Container with two integer values, and no additional attributes.

* `["^Hello world", {"#n": "hello"}]` - A Container named "hello" with the text object "Hello world".

* `["^test", {"subContainer": [5, 6, null]}]`

   A container with the text object "test" and a nested container named "subContainer" that resembles the first example.

## Values

Values are the main content objects. The most useful for written content is the String Value, which is used for all the main text within the story flow.

String Values are represented in JSON by preceding the text in quotes by a leading `^` to distinguish it from many other object types which have special names (for example, control commands and native functions). The only exception is a newline, which can be simply written as `"\n"`.

Values may also be used in logic/calculations, for example with the `int` and `float` types.

Supported types:

* **string**: Represented with a leading `^` to differentiate from other string-based objects. e.g. `"^Hello world"` is used in JSON to represent the text `Hello world`, and `"^^ up there ^"` would be the text `^ up there ^`. No `^` is needed for a newline, so it's just `"\n"`.
* **int**, **float**, and **bool**: these are represented using their standard JSON counterparts. e.g. `5`, `5.6`, `true`.
* **dynamic array value**: represented as a JSON array that is not a Container. A JSON array is parsed as a Container only when it has the Container terminator shape described above, such as a final `null` element or a final object containing Container metadata. Otherwise it is a value array. For example, `[1, 2.5, true, "^text", [3]]` is an array value containing an integer, float, boolean, string, and nested array.
* **dynamic object/struct value**: represented as a JSON object whose fields are runtime values. For example:

    ```json
    {
        "flags": [true, false],
        "hp": 10,
        "name": "^Ada"
    }
    ```

    This represents an object value with `flags`, `hp`, and `name` fields. Static ink-rs source type names and field declarations are not serialized in story JSON or save JSON in this phase; only the runtime values are serialized.
* **dynamic Dict value**: represented as an array marker that stores the key
  type and an array of key/value entries:

    ```json
    ["dict", "string", [["ada", 10], ["grace", "^compiler"]]]
    ```

    The second element is the key type, either `"string"` or `"int"`. The third
    element is an array of two-item `[key, value]` entries. String keys are
    plain JSON strings, int keys are JSON integer numbers, and entry values use
    the same recursive runtime value encoding as arrays and objects.

    ```json
    ["dict", "int", [[1, "^one"], [2, {"hp": 10}]]]
    ```

    Dicts use an array marker instead of a JSON object so the encoding does not
    consume the open field namespace used by dynamic object/struct values. This
    also keeps int keys as integer entry keys rather than converting them to
    JSON object field names. Ordinary object fields named `dict`, `^dict`, or
    `entries` remain dynamic object fields and are not interpreted as Dicts.
* **interface value**: represented as a runtime string containing the source
  module name, using the same string value encoding as other strings. For
  example, an `interface<IItem>` variable initialized with module `left` stores
  the string value `"left"`. Interface values keep this string shape in compiled
  story JSON and runtime save-state JSON; the compiled story's top-level
  `interfaces` metadata supplies the validation contract for dynamic dispatch.
* **divert target**: represents a variable divert target, for example as used in the following ink:

        -> somewhere

    Represented in runtime JSON as an object of the form: `{"^->": "path.to.target"}`

* **variable pointer**: used for references to variables, for example when declaring a function with the following ink:

        == function myFunction(ref x: int) => void ==

    Represented in runtime JSON as an object of the form: `{"^var": "varname", "ci": 0}`. Where `ci` is "context index", with the following possible values:

    * **-1** - default value, context yet to be determined.
    * **0**  - Variable is a global
    * **1 or more** - variable is a local/temporary in the callstack element with the given index.

## Void

Represented by `"void"`, this is used to place an object on the evaluation stack when a function returns without a value.

## Control commands

Control commands are special instructions to the text engine to perform various actions. They are all represented by a particular text string. This is the current compiled-story JSON command set, not a source syntax list.

* `"ev"` - Begin logical evaluation mode. In evaluation mode, objects that are encountered are added to an evaluation stack, rather than simply echoed into the main text output stream. As they're pushed onto the stack, they may be processed by other commands, functions, etc.
* `"/ev"` - End logical evaluation mode. Future objects will be appended to the output stream rather than to the evaluation stack.
* `"out"` - The topmost object on the evaluation stack is popped and appended to the output stream (main story output).
* `"pop"` - Pops a value from the evaluation stack, *without* appending to the output stream.
* `"->->"` and `"~ret"` pop the callstack - used for returning from a tunnel or function respectively. They are specified independently for error checking, since the callstack is aware of whether each element was pushed as a tunnel or function in the first place.
* `"du"` - Duplicate the topmost object on the evaluation stack. Useful since some commands consume objects on the evaluation stack.
* `"str"` - Begin string evaluation mode. Adds a marker to the output stream, and goes into content mode (from evaluation mode). Must have already been in evaluation mode when this is encountered. See below for explanation.
* `"/str"` - End string evaluation mode. All content after the previous Begin marker is concatenated together, removed from the output stream, and appended as a string value to the evaluation stack. Re-enters evaluation mode immediately afterwards.
* `"nop"` - No-operation. Does nothing, but is useful as an addressable piece of content to divert to.
* `"rnd"` - Pops maximum and minimum integer bounds from the evaluation stack, pushes a deterministic pseudo-random integer in that inclusive range, and advances the story random state.
* `"srnd"` - Pops an integer seed, stores it as the story random seed, resets the previous random value, and pushes `void`.

Natural story endings are represented by reaching the end of the current
container structure. Current compiled JSON has no terminal control command for
story end.

## Native functions

These are mathematical, logical, and dynamic-value functions that pop arguments from the evaluation stack, evaluate the result, and push the result back onto the evaluation stack. The following operators are supported:

`"+"`, `"-"`, `"/"`, `"*"`, `"%"` (mod), `"_"` (unary negate), `"=="`, `">"`, `"<"`, `">="`, `"<="`, `"!="`, `"!"` (unary 'not'), `"&&"`, `"||"`, `"MIN"`, `"MAX"`, `"to_str"`, `"FIELD"`, `"INDEX"`, `"SET_FIELD"`, `"SET_INDEX"`, `"LEN"`, `"ARRAY_REMOVE"`, `"ARRAY_PUSH"`, `"ARRAY_INSERT"`, `"DICT_HAS"`, `"DICT_SIZE"`, `"DICT_REMOVE"`, `"DICT_KEYS"`

`"to_str"` converts one typed runtime value to a string. `"FIELD"` and `"INDEX"` read object fields and array or Dict elements. `"SET_FIELD"` and `"SET_INDEX"` return updated object, array, or Dict values; assignment instructions store the updated value back into the target variable. `"LEN"` returns an array length as an integer. `"ARRAY_REMOVE"`, `"ARRAY_PUSH"`, and `"ARRAY_INSERT"` return updated array copies; assignment lowering stores them back into the source lvalue. `"DICT_HAS"`, `"DICT_SIZE"`, `"DICT_REMOVE"`, and `"DICT_KEYS"` provide typed Dict collection helpers.

Boolean values are represented as JSON booleans and runtime `bool` values. Some
numeric operations still accept C-style truthiness where non-zero integers or
floats are treated as true, but comparison and logical operations push boolean
results to the evaluation stack.

## Divert

Diverts can take the following forms:

* `{"->": "path.to.target"}` - a standard divert to content at a particular path.
* `{"->": "variableTarget", "var": true}` - as above, except that `var` specifies that the target is the name of a variable containing a *divert target* value.
* `{"f()": "path.to.func"}` - a function-call, which is defined as a divert that pushes an element to the callstack. Note that it doesn't necessarily correspond directly to an ink function, since choices use them internally too.
* `{"->t->": "path.tunnel"}` - a tunnel, which works similarly to a function call by pushing an element to the callstack. The only difference is that the callstack is aware of the type of element that was pushed, for error checking.
* `{"x()": "externalFuncName", "exArgs": 5}` - an external (game-side) function call, that optionally takes the specified number of arguments.
* `{"i->": "target", "interface": "IItem"}` - construct a dynamic divert
  target from an interface value already on the evaluation stack. The popped
  string value must name a compiled module that implements `IItem`; the runtime
  combines that module with member `target` and pushes a divert target value.
* `{"i()": "score", "interface": "IItem", "args": 1}` - dispatch a dynamic
  interface function call. The call arguments are evaluated first, then the
  interface value is evaluated, and the instruction uses `args` to consume the
  correct number of call arguments plus the interface value. The runtime
  validates the module implementation and member kind with top-level
  `interfaces` metadata before calling the resolved module function.

Additionally, a `"c"` property set to `true` indicates that the divert is conditional, and should therefore pop a value off the evaluation stack to determine whether the divert should actually happen.

Module external calls use source-qualified host binding names in the `"x()"`
field, for example `{"x()": "audio::play", "exArgs": 1}`. They are not runtime
container paths, so they keep the `::` separator.

This document describes compiled story JSON. Runtime save-state JSON remains
owned by the runtime layer; it serializes current runtime values, including
interface values as strings, but does not duplicate the compiled story's
`interfaces` metadata.

The current runtime save-state format is `inkSaveVersion` 3. It stores the
active continuation/callstack frames, global `variablesState`, deterministic
random state (`storySeed` and `previousRandom`), `inkSaveVersion`, and
`inkFormatVersion`. It does not serialize generated choices, choice
continuation snapshots, multi-flow maps, visit counts, turn indices, current
divert targets, or evaluation-stack internals. Calling `save_state()` while
choices are pending returns an error. Earlier saves and v3 saves containing
removed fields such as `currentChoices`, `choiceThreads`, `flows`,
`currentFlowName`, `evalStack`, `currentDivertTarget`, `visitCounts`,
`turnIndices`, or `resumeMode` are rejected rather than migrated.

## Variable assignment

Pops a value from the evaluation stack, and assigns it to a named variable, either globally or locally (in a `temp`, or a passed parameter). The `"re"` property being set indicates that it's a re-assignment rather than a brand new declaration.

Examples:

* `{"VAR=": "money", "re": true}` - Pop a value from the evaluation stack, and assign it to the already-declared global variable `money`.
* `{"VAR=": "state::money", "re": true}` - Assign to a module global. Module
  global variable names keep the `module::name` source form in compiled story
  JSON and save-state JSON.
* `{"temp=": "x"}` - Pop a value from the evaluation stack, and assign it to a newly declared temporary variable named `x`.

## Variable reference

Obtain the current value of a named variable, and push it to the evaluation stack.

Example:

* `{"VAR?": "danger"}` - Get an existing global or temporary variable named `danger` and push its value to the evaluation stack.
* `{"VAR?": "state::danger"}` - Get a module global by its source-qualified
  runtime variable name.

## ChoicePoint

Generates an instance of a `Choice`. Its exact behaviour depends on its flags. It doesn't contain any text itself, since choice text is generated at runtime and added to the evaluation stack. When a ChoicePoint is encountered, it pops content off the evaluation stack according to its flags, which indicate which texts are needed.

A ChoicePoint object's structure in JSON is:

```json
{
    "*": "path.when.chosen",
    "flg": 2
}
```

The path when chosen is the target path of a Container of content, and is assigned when calling `ChooseChoiceIndex`.

The `flg` field is a bitfield of flags:

 * **0x1 - Has condition?**: Set if the story should pop a value from the evaluation stack in order to determine whether a choice instance should be created at all.
 * **0x2 - Has start content?** - Choice display text should be popped from the evaluation stack.
 * **0x8 - Is invisible default?** - When this is enabled, the choice isn't provided to the game (isn't presented to the player), and instead is automatically followed if there are no other choices generated.

Only these bits are valid in current compiled JSON. Other bits are rejected.

Example of the full JSON output shape, including the ChoicePoint object, when
generating display text and chosen content from a current `* Hello` choice:

```jsonc
// Outer container
[

  // Evaluate choice text.
  // Starts by calling a "function" labelled 's', which is the start
  // content for the choice.
  "ev",
  "str",
  {
    "f()": ".^.s"
  },
  "/str",

  // Evaluation of choice text complete
  "/ev",

  // ChoicePoint object itself:
  //  - linked to own container named 'c'
  //  - Flag 2 means it has start content
  {
    "*": ".^.c",
    "flg": 2
  },

  // Named content from outer container - 's' and 'c'
  {
    // Inner container for start content is labelled 's'
    "s": [
      "^Hello",
      null
    ],

    // Inner container for content when choice is chosen
    // First repeats the start content ('s'),
    // before continuing.
    "c": [
      {
        "f()": "0.0.s"
      },
      "^, world.",
      "\n",
      null
    ]
  }
]
```

## Paths

Paths won't ever appear on their own in a Container, but are used by various objects (for example, see Diverts) to reference content within the hierarchy.

Paths are a dot-separated syntax:

    path.to.target

Where each element of the path references a sub-object, drilling down into the hierarchy.

However, paths can have several element types between the dots:

 * **Names** - to reference particular knots, stitches, gathers and named choices. These specify a named content object within a Container.
 * **Indices** - integers that specify the index of a content object within the ordered array section of a Container.
 * **Parent** - Denoted with a `^`. (Similar to using ".." in a file system.)

Relative paths *lead* with a dot rather than starting with a name or index.

Examples:

* `building.entrance.3.0` - the first element of a Container at the fourth element of a Container named `entrance` within a Container named `building` of the root Container.
* `items.take` - the runtime path for source flow `items::take`.
* `.^.1` - the second element of the parent Container.
