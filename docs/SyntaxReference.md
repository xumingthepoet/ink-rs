# Syntax Reference

<details>
  <summary>Table of Contents</summary>

  * [Introduction](#introduction)
  * [Current ink-rs Source Shape](#current-ink-rs-source-shape)
  * [Part One: The Basics](#part-one-the-basics)
    * [1) Content](#1-content)
    * [2) Choices](#2-choices)
    * [3) Knots](#3-knots)
    * [4) Diverts](#4-diverts)
    * [5) Branching The Flow](#5-branching-the-flow)
    * [6) Modules and Stitches](#6-modules-and-stitches)
    * [7) Varying Choices](#7-varying-choices)
    * [8) Variable Text](#8-variable-text)
    * [9) Game Queries and Functions](#9-game-queries-and-functions)
  * [Part 2: Weave](#part-2-weave)
    * [1) Gathers](#1-gathers)
    * [2) Nested Flow](#2-nested-flow)
    * [3) Tracking a Weave](#3-tracking-a-weave)
  * [Part 3: Variables and Logic](#part-3-variables-and-logic)
    * [1) Global Variables](#1-global-variables)
    * [2) Logic](#2-logic)
    * [3) Conditional blocks (if/else)](#3-conditional-blocks-ifelse)
    * [4) Temporary Variables](#4-temporary-variables)
    * [5) Functions](#5-functions)
    * [6) Constants](#6-constants)
    * [7) Advanced: Game-side logic](#7-advanced-game-side-logic)
   * [Part 4: Advanced Flow Control](#part-4-advanced-flow-control)
     * [1) Tunnels](#1-tunnels)
     * [2) Threads](#2-threads)
   * [Part 5: International character support in identifiers](#part-5-international-character-support-in-identifiers)
</details>

## Introduction

For a short current-language entry point, start with `LanguageOverview.md`.
This file is the full maintained syntax reference for ink-rs.

**ink** is a scripting language built around the idea of marking up pure-text with flow in order to produce interactive scripts.

At its most basic, it can be used to write a Choose Your Own-style story, or a branching dialogue tree. But its real strength is in writing dialogues with lots of options and lots of recombination of the flow.

**ink** offers several features to enable non-technical writers to branch often, and play out the consequences of those branches, in both minor and major ways, without fuss.

The script aims to be clean and logically ordered, so branching dialogue can be tested "by eye". The flow is described in a declarative fashion where possible.

It's also designed with redrafting in mind; so editing a flow should be fast.

## Current ink-rs Source Shape

Every runnable ink-rs story is built from explicit modules. A module starts
with `=== module name ===`; knots and functions inside modules use `==`; and
stitches still use `=`. Exactly one module in a compilation must define
`== main ==`, which is the story entry point.

Story content and tags belong inside knots or stitches, not at module level.
The first non-blank line of each source file must be a module or interface
declaration. The host compiler receives every source file as an explicit source
input. Each module name may appear only once in a compilation, so repeating the
same `=== module name ===` in another file is an error rather than a way to
extend that module. A module can use another module's knots, functions,
constants, globals, structs, or externals only after an explicit
`FROM module IMPORT ...`, and cross-module references use `module::symbol`.

For example:

	=== module game ===
	FROM shop IMPORT price, describe
	VAR gold: int = 5

	== main ==
	{shop::describe()}
	Gold: {gold}
	Price: {shop::price}
	~ shop::price += 2
	Updated price: {shop::price}
	-> END

	=== module shop ===
	VAR price: int = 3

	== function describe() => string ==
	~ return "The shop is open."

Many examples below show snippets that live inside a knot or stitch. A complete
runnable file still needs the explicit module and `main` structure shown above.

# Part One: The Basics

## 1) Content

### The simplest ink script

The most basic runnable ink-rs script contains an explicit module and a `main`
knot:

	=== module game ===
	== main ==
	Hello, world!
	-> END

On running, this will output the content, and then stop.

Inside a knot or stitch, text on separate lines produces new paragraphs:

	Hello, world!
	Hello?
	Hello, are you there?

produces output that looks the same.


### Comments

By default, text inside knots and stitches will appear in the output content,
unless specially marked up.

The simplest mark-up is a comment. **ink** supports two kinds of comment. There's the kind used for someone reading the code, which the compiler ignores:

	"What do you make of this?" she asked.

	// Something unprintable...

	"I couldn't possibly comment," I replied.

	/*
		... or an unlimited block of text
	*/

and there's the kind used for reminding the author what they need to do, that the compiler prints out during compilation:


	TODO: Confirm the arrival time.

### Tags

Text content from the game will appear 'as is' when the engine runs. However, it can sometimes be useful to mark up a line of content with extra information to tell the game what to do with that content.

**ink** provides a simple system for tagging lines of content, with hashtags.

	A line of normal game-text. # colour it blue

These don't show up in the main text flow, but can be read off by the game and used as you see fit.


## 2) Choices

Input is offered to the player via text choices. A text choice is indicated by
one or more `*` characters. Choices are repeatable unless an explicit condition
hides them.

Choice line text is display text. It is shown to the player as an option, but it is not printed again after the player selects it. Put selected response text in the indented body of the choice.

	Hello world!
	*	Hello back!
		Nice to hear from you!

This produces the following game:

	Hello world!
	1: Hello back!

	> 1
	Nice to hear from you!

Square brackets in choice lines are ordinary text characters except for the
dynamic-choice prefix described in [Varying Choices](#7-varying-choices). A
choice such as `* Ask [again]` displays as `Ask [again]`.

This is most useful when writing dialogue choices:

	"What's that?" my master asked.
	*	"I am somewhat tired."
		"I am somewhat tired," I repeated.
		"Really," he responded. "How deleterious."

produces:

	"What's that?" my master asked.
	1: "I am somewhat tired."
	> 1
	"I am somewhat tired," I repeated.
	"Really," he responded. "How deleterious."

### Multiple Choices

To make choices really choices, we need to provide alternatives. We can do this simply by writing them out:

	"What's that?" my master asked.
	*	"I am somewhat tired."
		"I am somewhat tired," I repeated.
		"Really," he responded. "How deleterious."
	*	"Nothing, Monsieur!"
		"Nothing, Monsieur!" I replied.
		"Very good, then."
	*  "I said, this journey is appalling."
		"I said, this journey is appalling." I want no more of it.
		"Ah," he replied, not unkindly. "I see you are feeling frustrated. Tomorrow, things will improve."

This produces the following game:

	"What's that?" my master asked.

	1: "I am somewhat tired."
	2: "Nothing, Monsieur!"
	3: "I said, this journey is appalling."

	> 3
	"I said, this journey is appalling and I want no more of it."
	"Ah," he replied, not unkindly. "I see you are feeling frustrated. Tomorrow, things will improve."

The above syntax is enough to write a single set of choices. In a real game, we'll want to move the flow from one point to another based on what the player chooses. To do that, we need to introduce a bit more structure.

## 3) Knots

### Pieces of content are called knots

To allow the game to branch we need to mark up sections of content with names (as an old-fashioned gamebook does with its 'Paragraph 18', and the like.)

These sections are called "knots" and they're the fundamental structural unit of ink content.

### Writing a knot

Inside a module, the start of a knot is indicated by two equals signs:

	== top_knot ==

(The equals signs on the end are optional; and the name needs to be a single word with no spaces.)

The start of a knot is a header; the content that follows will be inside that knot.

	== back_in_london ==

	We arrived into London at 9.45pm exactly.

#### Advanced: a knottier "hello world"

Every runnable ink-rs file starts with an explicit module, and the story starts
at that module's `main` knot. If you want `main` to hand off to another knot,
use a divert arrow `->`, which is covered properly in the next section.

The simplest knotty script is:

	=== module game ===

	== main ==
	-> top_knot

	== top_knot ==
	Hello world!

However, **ink** doesn't like loose ends, and produces diagnostics or runtime
errors when it thinks this has happened. The `top_knot` above has no choice,
divert, `-> DONE`, or `-> END` after its content, so the flow can run out.

The following plays and compiles without error:

	=== module game ===

	== main ==
	-> top_knot

	== top_knot ==
	Hello world!
	-> END

`-> END` is a marker for both the writer and the compiler; it means "the story flow should now stop".

## 4) Diverts

### Knots divert to knots

You can tell the story to move from one knot to another using `->`, a "divert arrow". Diverts happen immediately without any user input.

	== back_in_london ==

	We arrived into London at 9.45pm exactly.
	-> hurry_home

	== hurry_home ==
	We hurried home to Savile Row as fast as we could.

#### Diverts are invisible

Diverts are intended to be seamless and can even happen mid-sentence:

	== hurry_home ==
	We hurried home to Savile Row -> as_fast_as_we_could

	== as_fast_as_we_could ==
	as fast as we could.

produces the same line as above:

	We hurried home to Savile Row as fast as we could.

#### Glue

The default behaviour inserts line-breaks before every new line of content. In some cases, however, content must insist on not having a line-break, and it can do so using `<>`, or "glue".

	== hurry_home ==
	We hurried home <>
	-> to_savile_row

	== to_savile_row ==
	to Savile Row
	-> as_fast_as_we_could

	== as_fast_as_we_could ==
	<> as fast as we could.

also produces:

	We hurried home to Savile Row as fast as we could.

You can't use too much glue: multiple glues next to each other have no additional effect. (And there's no way to "negate" a glue; once a line is sticky, it'll stick.)


## 5) Branching The Flow

### Basic branching

Combining knots, options and diverts gives us the basic structure of a choose-your-own game.

	== paragraph_1 ==
	You stand by the wall of Analand, sword in hand.
	* Open the gate -> paragraph_2
	* Smash down the gate -> paragraph_3
	* Turn back and go home -> paragraph_4

	== paragraph_2 ==
	You open the gate, and step out onto the path.

	...

### Branching and joining

Using diverts, the writer can branch the flow, and join it back up again, without showing the player that the flow has rejoined.

	== back_in_london ==

	We arrived into London at 9.45pm exactly.

	*	"There is not a moment to lose!" I declared.
		-> hurry_outside

	*	"Monsieur, let us savour this moment!" I declared.
		My master clouted me firmly around the head and dragged me out of the door.
		-> dragged_outside

	*	We hurried home -> hurry_outside


	== hurry_outside ==
	We hurried home to Savile Row -> as_fast_as_we_could


	== dragged_outside ==
	He insisted that we hurried home to Savile Row
	-> as_fast_as_we_could


	== as_fast_as_we_could ==
	<> as fast as we could.


### The story flow

Knots and diverts combine to create the basic story flow of the game. This flow is "flat" - there's no call-stack, and diverts aren't "returned" from.

In ink-rs, the story flow starts at the unique `main` knot, bounces around in a
spaghetti-like mess, and eventually, hopefully, reaches a `-> END`.

The very loose structure means writers can get on and write, branching and rejoining without worrying about the structure that they're creating as they go. There's no boiler-plate to creating new branches or diversions, and no need to track any state.

#### Advanced: Loops

You absolutely can use diverts to create looped content. In ink-rs, use
explicit variables, conditions, and functions to vary content or hide choices
inside those loops.

See the sections on [Varying Text](#8-variable-text) and [Conditional Choices](#conditional-choices) for more information.

Oh, and the following is legal and not a great idea:

	== round ==
	and
	-> round

## 6) Modules and Stitches

### Modules organize source files

Modules are the top-level namespace for ink-rs source. A compilation can use
modules declared in one file or in multiple source inputs supplied by the host
compiler API, but module names are unique per compilation. The compiler does
not merge same-named modules across files; declaring `=== module travel ===` in
two different source inputs is a duplicate-module error. The caller passes
every source file explicitly.

	=== module travel ===
	FROM shop IMPORT ticket_price

	== main ==
	We boarded the train.
	Ticket price: {shop::ticket_price}
	-> the_orient_express

	== the_orient_express ==
	We were underway.
	-> END

	=== module shop ===
	VAR ticket_price: int = 3

Imports are exact allow-lists. Importing `ticket_price` from `shop` permits
`shop::ticket_price`; it does not make `ticket_price` visible unqualified, and
it does not re-export anything imported by `shop`.

Use a comma-separated import list when a module exposes multiple symbols:

	FROM shop IMPORT ticket_price, route_name, describe_stop, route_status, next_departure

A bare module import creates a module dependency without authorizing any static
symbol access:

	FROM shop

Modules can declare explicit interface implementations in the module header:

	=== interface IRoute ===
	== destination ==

	=== module travel implements IRoute ===
	== destination ==
	-> END

Interfaces declare signatures only. A knot signature requires an implementing
module to provide a knot with the same name and parameters. A function
signature requires an implementing function with the same parameters and return
type. A module can implement more than one interface:

	=== module express implements IRoute, IInspectable ===

An interface value stores the selected implementation module for a declared
interface. Use `interface<Name>` in type positions:

	VAR route: interface<IRoute> = left
	VAR backups: interface<IRoute>[] = [left, right]

Module names used as interface values are module literals. The current module
must import each implementation module with a bare module import:

	FROM left
	FROM right

`FROM right IMPORT arrive` authorizes static `right::arrive` references, but it
does not authorize `right` as an interface value. Use both import forms when a
module is needed both as a value and as a static symbol source.

Dynamic interface knot targets use a braced interface expression followed by
`::member`. The dynamic divert form wraps that target expression in the normal
dynamic-divert braces:

	-> {{route}::destination}
	-> {{route}::arrive}(2)

Dynamic interface function calls are ordinary expressions:

	{route}::fare(base_price)

For example:

	=== interface IRoute ===
	== arrive(stops: int) ==
	== function fare(base: int) => int ==

	=== module game ===
	FROM left
	FROM right
	FROM rates IMPORT base

	VAR route: interface<IRoute> = left

	== main ==
	Default fare {current_fare()}.
	~ route = right
	Switched fare {current_fare()}.
	-> {{route}::arrive}(2)

	== function current_fare() => int ==
	~ return {route}::fare(rates::base)

	=== module rates ===
	VAR base: int = 3

	=== module left implements IRoute ===
	== arrive(stops: int) ==
	Left route {stops}.
	-> END

	== function fare(base: int) => int ==
	~ return base + 1

	=== module right implements IRoute ===
	== arrive(stops: int) ==
	Right route {stops}.
	-> END

	== function fare(base: int) => int ==
	~ return base + 2

Interface values are string-backed runtime values containing the implementation
module name. They can be assigned, stored in arrays and structs, and saved in
the existing save JSON string/array/object shapes. Dynamic access validates at
use time that the stored module name implements the required interface member.

### Knots can be subdivided

As stories get longer, they become more confusing to keep organised without some additional structure.

Knots can include sub-sections called "stitches". These are marked using a single equals sign.

	=== module travel ===
	== the_orient_express ==
	= in_first_class
		...
	= in_third_class
		...
	= in_the_guards_van
		...
	= missed_the_train
		...

One could use a knot for a scene, for instance, and stitches for the events within the scene.

### Stitches have unique names

A stitch can be diverted to using its "address".

	*	Travel in third class
		-> the_orient_express.in_third_class

	*	Travel in the guard's van
		-> the_orient_express.in_the_guards_van

### The first stitch is the default

Diverting to a knot which contains stitches will divert to the first stitch in the knot. So:

	*	Travel in first class
		"First class, Monsieur. Where else?"
		-> the_orient_express

is the same as:

	*	Travel in first class
		"First class, Monsieur. Where else?"
		-> the_orient_express.in_first_class

(...unless we move the order of the stitches around inside the knot!)

You can also include content at the top of a knot outside of any stitch. However, you need to remember to divert out of it - the engine *won't* automatically enter the first stitch once it's worked its way through the header content.

	== the_orient_express ==

	We boarded the train, but where?
	*	First class -> in_first_class
	*	Second class -> in_second_class

	= in_first_class
		...
	= in_second_class
		...


### Local diverts

From inside a knot, you don't need to use the full address for a stitch.

	-> the_orient_express

	== the_orient_express ==
	= in_first_class
		I settled my master.
		*	Move to third class
			-> in_third_class

	= in_third_class
		I put myself in third.

This means stitches and knots can't share names, but several knots can contain the same stitch name. (So both the Orient Express and the SS Mongolia can have first class.)

The compiler will warn you if ambiguous names are used.

## 7) Varying Choices

### Choices repeat unless conditions hide them

In ink-rs, choices are repeatable. If a loop returns to the same choice point,
the same authored choices can appear again. When a choice should disappear,
track that state explicitly and put a condition on the choice:

	VAR asked_hat: bool = false
	VAR asked_briefcase: bool = false

	== find_help ==

		You search desperately for a friendly face in the crowd.
		*	{ not asked_hat } The woman in the hat?
			~ asked_hat = true
			The woman in the hat pushes you roughly aside. -> find_help
		*	{ not asked_briefcase } The man with the briefcase?
			~ asked_briefcase = true
			The man with the briefcase looks disgusted as you stumble past him. -> find_help

#### Fallback choices

Fallback choices are useful when every visible choice can be hidden by
conditions. They are never displayed to the player, but are chosen by the game
if no other options exist.

A fallback choice is simply a "choice without choice text":

	*	-> out_of_options

And, in a slight abuse of syntax, we can make a default choice with content in it, using an "choice then arrow":

	* 	->
		Mulder never could explain how he got out of that burning box car. -> season_2

#### Example of a fallback choice

Adding this into the previous example gives the story an explicit route when
both visible options have been hidden:

	== find_help ==

		You search desperately for a friendly face in the crowd.
		*	{ not asked_hat } The woman in the hat?
			~ asked_hat = true
			The woman in the hat pushes you roughly aside. -> find_help
		*	{ not asked_briefcase } The man with the briefcase?
			~ asked_briefcase = true
			The man with the briefcase looks disgusted as you stumble past him. -> find_help
		*	->
			But it is too late: you collapse onto the station platform. This is the end.
			-> END


### Repeatable choices

In ink-rs, choices are repeatable. Revisiting the same choice point can show
the same authored choices again. Use conditions or explicit variables when a
choice should disappear.

	== homers_couch ==
		*	Eat another donut
			You eat another donut. -> homers_couch
		*	Get off the couch
			You struggle up off the couch to go and compose epic poetry.
			-> END

Fallback choices can repeat too.

	== conversation_loop ==
		*	Talk about the weather -> chat_weather
		*	Talk about the children -> chat_children
		*	-> sit_in_silence_again

### Dynamic Choices From Arrays

A choice can expand at runtime from an array by putting a binding prefix
immediately after the `*` marker:

	VAR options: string[] = ["Alpha", "Hidden", "Beta"]
	VAR enabled: bool[] = [true, false, true]

	== main ==
		* Fixed first
			You picked the fixed first option.
			-> DONE
		* [i, option in options] {enabled[i]}: {i}: {option}
			You picked {i}: {option}.
			-> DONE
		* Fixed last
			You picked the fixed last option.
			-> DONE

The binding can name only the item, as in `[option in options]`, or both the
zero-based index and item, as in `[i, option in options]`. The expression after
`in` must have an array type. The array expression is evaluated once before
choices are generated; each generated choice then receives its own index and
item values.

Dynamic choices behave like ordinary choices after expansion. They may use any
choice depth marker such as `*`, `**`, or `***`; they may mix with static
choices; conditions filter individual generated choices; fallback choices still
run if no visible choices remain; and generated choices save/load with their
captured runtime state.

The dynamic variables are visible in the choice condition, displayed choice
text, selected-choice body, tags, and nested choices:

	VAR topics: string[] = ["East", "West"]
	VAR details: string[][] = [["E1", "E2"], ["W1"]]

	== main ==
		* Topic menu
			** [i, topic in topics] Topic {i}: {topic}
				*** [detail in details[i]] Detail {topic}: {detail}
					You picked {topic}: {detail}.
					-> DONE

Static choices and gathers can still use `(label)` labels. Dynamic choices do
not support labels; use item data, explicit state, or gather labels when a
generated option needs an addressable outcome.

### Conditional Choices

You can also turn choices on and off by hand. Conditions are still supported and should be based on explicit variables or typed expressions.

For example:

	VAR has_visited_paris: bool = false
	VAR met_estelle: bool = false

	*	{ not has_visited_paris } 	Go to Paris -> visit_paris
	* 	{ has_visited_paris } 		Return to Paris -> visit_paris

	*	{ met_estelle } Telephone Mme Estelle -> phone_estelle

Use explicit variables when a condition needs to remember authored state.

#### Advanced: multiple conditions

You can use several logical tests on an option; if you do, *all* the tests must all be passed for the option to appear. Adjacent condition blocks are evaluated as separate tests, so every block runs even if an earlier block is false.

	*	{ not has_visited_paris } 	Go to Paris -> visit_paris
	* 	{ has_visited_paris } { not bored_of_paris }
		Return to Paris -> visit_paris

When the visible choice text itself starts with a dynamic expression, put a
colon after the condition prefix to make the boundary explicit:

	*	{ has_key }: {locked_door_label} -> open_door
	*	{ has_visited_paris } { not bored_of_paris }: {return_label} -> visit_paris

#### Logical operators: AND and OR

Inside a single expression, ink-rs supports `and` (also written as `&&`) and `or` (also written as `||`) in the usual way, as well as brackets.

	*	{ not (visited_paris or visited_rome) && (visited_london || visited_new_york) } Wait. Go where? I'm confused. -> visit_someplace

For non-programmers `X and Y` means both X and Y must be true. `X or Y` means either or both. We don't have a `xor`.

Explicit `and` / `&&` and `or` / `||` short-circuit. `X and Y` skips `Y` when `X` is false; `X or Y` skips `Y` when `X` is true. This makes guard expressions safe:

	{ index < LEN(items) && items[index] == target }

Short-circuiting only applies inside the expression. Separate choice condition blocks such as `{a}{b}` are still separate tests, and both blocks are evaluated.

You can also use the standard `!` for `not`, though we recommend using `not` because it reads more clearly in prose-heavy source.

#### Tracking authored state in conditions

Conditions use explicit variables or expressions:

	VAR has_seen_clue: bool = false

	*	{has_seen_clue} Accuse Mr Jefferson


#### Advanced: more logic

**ink** supports a lot more logic and conditionality than covered here - see the section on [variables and logic](#part-3-variables-and-logic).


## 8) Variable Text

### Text can vary

So far, all the content we've seen has been static, fixed pieces of text. But content can also vary at the moment of being printed.

### Varying text with state

Use typed variables and conditional text to model text progression explicitly:

	VAR radio_step: int = 0

	{ if:
	- radio_step == 0:
	    Three!
	- radio_step == 1:
	    Two!
	- else:
	    One!
	}
	~ radio_step = radio_step + 1

For random variation, keep the random choice in explicit story or host state
using `RANDOM`, `SEED_RANDOM`, or a typed variable.


### Conditional Text

Text can also vary depending on logical tests, just as options can.

	{met_blofeld: "I saw him. Only for a moment." }

and

	"His real name was {met_blofeld.learned_his_name: Franz|a secret}."

These can appear as separate lines, or within a section of content. They can even be nested, so:

	{met_blofeld: "I saw him. Only for a moment. His real name was {met_blofeld.learned_his_name: Franz|kept a secret}." | "I missed him. Was he particularly evil?" }

can produce either:

	"I saw him. Only for a moment. His real name was Franz."

or:

	"I saw him. Only for a moment. His real name was kept a secret."

or:

	"I missed him. Was he particularly evil?"

## 9) Game Queries and Functions

**ink-rs** provides builtin functions for random numbers, numeric conversion, math, and typed collection helpers.

The convention is to name these in capital letters.

### Author state uses variables

Use variables for authored state that needs to persist across choices or knots:

	VAR choices_seen: int = 0
	VAR has_slept: bool = false

	*	{not has_slept} Sleep
		~ has_slept = true
		You sleep.
		-> END

### SEED_RANDOM()

For testing purposes, it's often useful to fix the random number generator so ink will produce the same outcomes every time you play. You can do this by "seeding" the random number system.

	~ SEED_RANDOM(235)

The number you pass to the seed function is arbitrary, but providing different seeds will result in different sequences of outcomes.

#### Advanced: more queries

You can make your own external functions, though the syntax is a bit different: see the section on [functions](#5-functions) below.


# Part 2: Weave

So far, we've been building branched stories in the simplest way, with "options" that link to "pages".

But this requires us to uniquely name every destination in the story, which can slow down writing and discourage minor branching.

**ink** has a much more powerful syntax available, designed for simplifying story flows which have an always-forwards direction (as most stories do, and most computer programs don't).

This format is called "weave", and its built out of the basic content/option syntax with two new features: the gather mark, `-`, and the nesting of choices and gathers.

## 1) Gathers

### Gather points gather the flow back together

Let's go back to the first multi-choice example at the top of this document.

	"What's that?" my master asked.
		*	"I am somewhat tired."," I repeated.
			"Really," he responded. "How deleterious."
		*	"Nothing, Monsieur!" I replied.
		*  "I said, this journey is appalling." and I want no more of it."
			"Ah," he replied, not unkindly. "I see you are feeling frustrated. Tomorrow, things will improve."

In a real game, all three of these options might well lead to the same conclusion - Monsieur Fogg leaves the room. We can do this using a gather, without the need to create any new knots, or add any diverts.

	"What's that?" my master asked.
		*	"I am somewhat tired."," I repeated.
			"Really," he responded. "How deleterious."
		*	"Nothing, Monsieur!" I replied.
			"Very good, then."
		*  "I said, this journey is appalling." and I want no more of it."
		"Ah," he replied, not unkindly. "I see you are feeling frustrated. Tomorrow, things will improve."

	-	With that Monsieur Fogg left the room.

This produces the following playthrough:

	"What's that?" my master asked.

	1: "I am somewhat tired."
	2: "Nothing, Monsieur!"
	3: "I said, this journey is appalling."

	> 1
	"I am somewhat tired," I repeated.
	"Really," he responded. "How deleterious."
	With that Monsieur Fogg left the room.

### Options and gathers form chains of content

We can string these gather-and-branch sections together to make branchy sequences that always run forwards.

	== escape ==
	I ran through the forest, the dogs snapping at my heels.

		* 	I checked the jewels were still in my pocket, and the feel of them brought a spring to my step. <>

		*  I did not pause for breath but kept on running. <>

		*	I cheered with joy. <>

	- 	The road could not be much further! Mackie would have the engine running, and then I'd be safe.

		*	I reached the road and looked about. And would you believe it?
		* 	I should interrupt to say Mackie is normally very reliable. He's never once let me down. Or rather, never once, previously to that night.

	-	The road was empty. Mackie was nowhere to be seen.

This is the most basic kind of weave. The rest of this section details  additional features that allow weaves to nest, contain side-tracks and diversions, divert within themselves, and above all, reference earlier choices to influence later ones.

#### The weave philosophy

Weaves are more than just a convenient encapsulation of branching flow; they're also a way to author more robust content. The `escape` example above has already four possible routes through, and a more complex sequence might have lots and lots more. Using normal diverts, one has to check the links by chasing the diverts from point to point and it's easy for errors to creep in.

With a weave, the flow is guaranteed to start at the top and "fall" to the bottom. Flow errors are impossible in a basic weave structure, and the output text can be easily skim read. That means there's no need to actually test all the branches in game to be sure they work as intended.

Weaves also allow for easy redrafting of choice-points; in particular, it's easy to break a sentence up and insert additional choices for variety or pacing reasons, without having to re-engineer any flow.


## 2) Nested Flow

The weaves shown above are quite simple, "flat" structures. Whatever the player does, they take the same number of turns to get from top to bottom. However, sometimes certain choices warrant a bit more depth or complexity.

For that, we allow weaves to nest.

This section comes with a warning. Nested weaves are very powerful and very compact, but they can take a bit of getting used to!

### Options can be nested

Consider the following scene:

	- 	"Well, Poirot? Murder or suicide?"
	*	"Murder!"
	* 	"Suicide!"
	-	Ms. Christie lowered her manuscript a moment. The rest of the writing group sat, open-mouthed.

The first choice presented is "Murder!" or "Suicide!". If Poirot declares a suicide, there's no more to do, but in the case of murder, there's a follow-up question needed - who does he suspect?

We can add new options via a set of nested sub-choices. We tell the script that these new choices are "part of" another choice by using two asterisks, instead of just one.


	- 	"Well, Poirot? Murder or suicide?"
		*	"Murder!"
			"And who did it?"
			* * 	"Detective-Inspector Japp!"
			* * 	"Captain Hastings!"
			* * 	"Myself!"
		* 	"Suicide!"
		-	Mrs. Christie lowered her manuscript a moment. The rest of the writing group sat, open-mouthed.

(Note that it's good style to also indent the lines to show the nesting, but the compiler doesn't mind.)

And should we want to add new sub-options to the other route, we do that in similar fashion.

	- 	"Well, Poirot? Murder or suicide?"
		*	"Murder!"
			"And who did it?"
			* * 	"Detective-Inspector Japp!"
			* * 	"Captain Hastings!"
			* * 	"Myself!"
		* 	"Suicide!"
			"Really, Poirot? Are you quite sure?"
			* * 	"Quite sure."
			* *		"It is perfectly obvious."
		-	Mrs. Christie lowered her manuscript a moment. The rest of the writing group sat, open-mouthed.

Now, that initial choice of accusation will lead to specific follow-up questions - but either way, the flow will come back together at the gather point, for Mrs. Christie's cameo appearance.

But what if we want a more extended sub-scene?

### Gather points can be nested too

Sometimes, it's not a question of expanding the number of options, but having more than one additional beat of story. We can do this by nesting gather points as well as options.

	- 	"Well, Poirot? Murder or suicide?"
			*	"Murder!"
				"And who did it?"
				* * 	"Detective-Inspector Japp!"
				* * 	"Captain Hastings!"
				* * 	"Myself!"
				- - 	"You must be joking!"
				* * 	"Mon ami, I am deadly serious."
				* *		"If only..."
			* 	"Suicide!"
				"Really, Poirot? Are you quite sure?"
				* * 	"Quite sure."
				* *		"It is perfectly obvious."
			-	Mrs. Christie lowered her manuscript a moment. The rest of the writing group sat, open-mouthed.

If the player chooses the "murder" option, they'll have two choices in a row on their sub-branch - a whole flat weave, just for them.

#### Advanced: What gathers do

Gathers are hopefully intuitive, but their behaviour is a little harder to put into words: in general, after an option has been taken, the story finds the next gather down that isn't on a lower level, and diverts to it.

The basic idea is this: options separate the paths of the story, and gathers bring them back together. (Hence the name, "weave"!)


### You can nest as many levels are you like

Above, we used two levels of nesting; the main flow, and the sub-flow. But there's no limit to how many levels deep you can go.

	-	"Tell us a tale, Captain!"
		*	"Very well, you sea-dogs. Here's a tale..."
			* * 	"It was a dark and stormy night..."
					* * * 	"...and the crew were restless..."
							* * * *  "... and they said to their Captain..."
									* * * * *		"...Tell us a tale Captain!"
		*	"No, it's past your bed-time."
	-	To a man, the crew began to yawn.

After a while, this sub-nesting gets hard to read and manipulate, so it's good style to divert away to a new stitch if a side-choice goes unwieldy.

But, in theory at least, you could write your entire story as a single weave.

### Example: a conversation with nested nodes

Here's a longer example:

	- I looked at Monsieur Fogg
	*	... and I could not contain myself.
		'What is the purpose of our journey, Monsieur?'
		'A wager,' he replied.
		* * 	'A wager!' I returned.
				He nodded.
				* * * 	'But surely that is foolishness!'
				* * *  'A most serious matter then!'
				- - - 	He nodded again.
				* * *	'But can we win?'
						'That is what we will endeavour to find out,' he answered.
				* * *	'A modest wager, I trust?'
						'Twenty thousand pounds,' he replied, quite flatly.
				* * * 	I asked nothing further of him then., and after a final, polite cough, he offered nothing more to me. <>
		* * 	'Ah.',' I replied, uncertain what I thought.
		- - 	After that, <>
	*	... but I said nothing and <>
	- we passed the day in silence.
	- -> END

with a couple of possible playthroughs. A short one:

	I looked at Monsieur Fogg

	1: ... and I could not contain myself.
	2: ... but I said nothing

	> 2
	... but I said nothing and we passed the day in silence.

and a longer one:

	I looked at Monsieur Fogg

	1: ... and I could not contain myself.
	2: ... but I said nothing

	> 1
	... and I could not contain myself.
	'What is the purpose of our journey, Monsieur?'
	'A wager,' he replied.

	1: 'A wager!'
	2: 'Ah.'

	> 1
	'A wager!' I returned.
	He nodded.

	1: 'But surely that is foolishness!'
	2: 'A most serious matter then!'

	> 2
	'A most serious matter then!'
	He nodded again.

	1: 'But can we win?'
	2: 'A modest wager, I trust?'
	3: I asked nothing further of him then.

	> 2
	'A modest wager, I trust?'
	'Twenty thousand pounds,' he replied, quite flatly.
	After that, we passed the day in silence.

Hopefully, this demonstrates the philosophy laid out above: that weaves offer a compact way to offer a lot of branching, a lot of choices, but with the guarantee of getting from beginning to end!


## 3) Tracking a Weave

Sometimes, the weave structure is sufficient. But when it's not, we need a bit more control.

### Weaves are largely unaddressed

By default, lines of content in a weave don't have an address or label, which means they can't be diverted to, and they can't be tested for. In the most basic weave structure, choices vary the path the player takes through the weave and what they see, but once the weave is finished those choices and that path are forgotten.

But should we want to remember what the player has seen, we can - we add in labels where they're needed using the `(label_name)` syntax.

### Gathers and options can be labelled

Gather points at any nested level can be labelled using parentheses.

	-  (top)

Once labelled, gather points can be diverted to, or tested for in conditionals, just like knots and stitches. This means you can use previous decisions to alter later outcomes inside the weave, while still keeping all the advantages of a clear, reliable forward-flow.

Options can also be labelled, just like gather points, using parentheses.
Labels come before conditions in the line.

These addresses can be used in conditional tests, which can be useful for creating options unlocked by other options.

	== meet_guard ==
	The guard frowns at you.

	* 	(greet) Greet him
		'Greetings.'
	*	(get_out) 'Get out of my way.',' you tell the guard.

	- 	'Hmm,' replies the guard.

	*	{greet} 	'Having a nice day?' // only if you greeted him

	* 	'Hmm?' you reply.

	*	{get_out} Shove him aside 	 // only if you threatened him
		You shove him sharply. He stares in reply, and draws his sword!
		-> fight_guard 			// this route diverts out of the weave

	-	'Mff,' the guard replies, and then offers you a paper bag. 'Toffee?'


### Scope

Inside the same block of weave, you can simply use the label name; from outside the block you need a path, either to a different stitch within the same knot:

	== knot ==
	= stitch_one
		- (gatherpoint) Some content.
	= stitch_two
		*	{stitch_one.gatherpoint} Option

or pointing into another knot:

	== knot_one ==
	-	(gather_one)
		* {knot_two.stitch_two.gather_two} Option

	== knot_two ==
	= stitch_two
		- (gather_two)
			*	{knot_one.gather_one} Option


#### Advanced: all options can be labelled

In truth, all content in ink is a weave, even if there are no gathers in sight. That means you can label *any* static option in the game with a parenthesized label, and then reference it using the addressing syntax. In particular, this means you can test *which* option a player took to reach a particular outcome.

	== fight_guard ==
	...
	= throw_something
	*	(rock) Throw rock at guard -> throw
	* 	(sand) Throw sand at guard -> throw

	= throw
	You hurl {throw_something.rock:a rock|a handful of sand} at the guard.


#### Advanced: Loops in a weave

Labelling allows us to create loops inside weaves. Here's a standard pattern for asking questions of an NPC.

	VAR guard_question_loops: int = 0

	- (opts)
		*	'Can I get a uniform from somewhere?' you ask the cheerful guard.
			'Sure. In the locker.' He grins. 'Don't think it'll fit you, though.'
		*	'Tell me about the security system.'
			'It's ancient,' the guard assures you. 'Old as coal.'
		*	'Are there dogs?'
			'Hundreds,' the guard answers, with a toothy grin. 'Hungry devils, too.'
		// We require the player to ask at least one question
		*	{loop} Enough talking
			-> done
	- (loop)
		~ guard_question_loops = guard_question_loops + 1
		{ if guard_question_loops < 3:
			-> opts
		- else:
			He scratches his head.
			'Well, can't stand around talking all day,' he declares.
		}
	- (done)
		You thank the guard, and move away.





#### Advanced: diverting to options

Options can also be diverted to: the divert goes to the output of having chosen that choice, as though the choice had been chosen. In ink-rs, choices are repeatable and choice-line text is display-only, so the authored body of the choice is what runs.

	- (opts)
	*	Pull a face
		You pull a face, and the soldier comes at you! -> shove

	*	(shove) Shove the guard aside
		You shove the guard to one side, but he comes back swinging.

	*	{shove} Grapple and fight -> fight_the_guard

	- 	-> opts

produces:

	1: Pull a face
	2: Shove the guard aside

	> 1
	You pull a face, and the soldier comes at you! You shove the guard to one side, but he comes back swinging.

	1: Grapple and fight

	>

#### Advanced: Gathers directly after an option

The following is valid, and frequently useful.

	*	"Are you quite well, Monsieur?" I asked.
		- - (quitewell) "Quite well," he replied.
	*	"How did you do at the crossword, Monsieur?" I asked.
		-> quitewell
	*	I said nothing and neither did my Master.
	-	We fell into companionable silence once more.

Note the level 2 gather point directly below the first option: there's nothing to gather here, really, but it gives us a handy place to divert the second option to.






# Part 3: Variables and Logic

So far we've made conditional text and conditional choices using explicit
variables, labels, and expressions.

**ink** also supports variables, both temporary and global, storing typed values such as numbers, booleans, strings, enums, structs, arrays, and dictionaries. It is fully-featured in terms of logic, and contains a few additional structures to help keep the often complex logic of a branching story better organised.


## 1) Global Variables

The most powerful kind of variable, and arguably the most useful for a story, is a variable to store some unique property about the state of the game - anything from the amount of money in the protagonist's pocket, to a value representing the protagonist's state of mind.

This kind of variable is "global" runtime state, but in ink-rs it still belongs
to a module namespace. Code in the same module can read and write it by
unqualified name. Code in another module must import it and then use the
qualified form, such as `state::gold`.

### Defining Global Variables

Global variables are defined with `VAR` at module top level, after the module
header and imports and outside knots, stitches, functions, choices, and
conditionals. Every `VAR` declaration must include an explicit
type using `name: Type`. A declaration may include an initializer, or omit it to
use the type's default value.

	=== module state ===
	VAR knowledge_of_the_cure: bool = false
	VAR players_name: string = "Emilia"
	VAR number_of_infected_people: int = 521
	VAR infection_ratio: float = 0.25
	VAR discovered_clues: string[] = ["ticket", "cipher"]
	VAR unopened_doors: int[]
	VAR clue_scores: Dict<string, int> = %{"ticket": 2}
	VAR retreat: -> = -> everybody_dies
	VAR checkpoints: ->[] = [-> the_train]

The primitive source types are `int`, `float`, `bool`, `string`, and `->`. The `->` type stores a divert target value such as `-> knot` or `-> knot.stitch`. Array types are written as `T[]`, so `string[]` means an array of strings, `int[][]` means an array of integer arrays, and `->[]` means an array of divert targets. Dict types are written as `Dict<string, V>` or `Dict<int, V>`, where `V` is any supported value type. Empty array and Dict literals are valid when the expected type is known, such as in `VAR unopened_doors: int[] = []` or `VAR clue_scores: Dict<string, int> = %{}`.

Long `VAR` and `CONST` initializers can spread array and struct literals across
multiple lines:

	VAR party: Player[] = [
		%Player{
			name: "Ada",
			stats: %Stats{
				hp: 10,
				ready: true
			}
		},
		%Player{
			name: "Bea",
			stats: %Stats{
				hp: 8,
				ready: false
			}
		}
	]

This multiline form is for module-level `VAR` and `CONST` declarations. Temporary
declarations such as `~ temp player: Player = ...` are still single-line logic
statements.

### Structs and struct values

Story-specific value shapes can be declared with `STRUCT`. Struct fields also require explicit types.

	STRUCT Stats {
		hp: int
		ready: bool
	}

	STRUCT Player {
		name: string
		stats: Stats
	}

	VAR current_player: Player = %Player{
		name: "Ada",
		stats: %Stats{
			hp: 10,
			ready: true
		}
	}

	VAR party: Player[] = [
		%Player{
			name: "Ada",
			stats: %Stats{
				hp: 10,
				ready: true
			}
		},
		%Player{
			name: "Bea",
			stats: %Stats{
				hp: 8,
				ready: false
			}
		}
	]

Struct literals always include the explicit struct type name, such as
`%Player{...}` or `%Player{}`. They can omit fields whose type has a default
value; fields of type `->` must be provided explicitly. For multi-field struct
values, put each literal field on its own line and separate literal fields with
commas. Struct declarations themselves stay one field per line without commas.
Fields and array items can be read or assigned with normal logic lines:

	{current_player.stats.hp}
	~ current_player.stats.hp += 1
	~ party[1].stats.hp += 1

### Dictionaries

Use `Dict<string, V>` or `Dict<int, V>` for typed key/value maps. The key type
is part of the value's runtime shape, so integer keys are not converted to
string field names. The value type can be primitive, enum, struct, array,
another Dict, or any other supported value type.

	VAR scores: Dict<string, int> = %{"ada": 10, "grace": 11}
	VAR names: Dict<int, string> = %{1: "one"}
	VAR nested: Dict<string, Dict<int, string>> = %{"ada": %{1: "ready"}}
	VAR empty_scores: Dict<string, int>

String-key Dict literals use quoted string keys. Int-key Dict literals use
integer keys. A single Dict literal cannot mix string and int keys. Empty `%{}`
is valid only when the expected type is known, such as
in a typed declaration, assignment to a typed variable, or typed return context. An
omitted Dict initializer defaults to an empty Dict with the declared key type.

Read and write entries with index syntax:

	{scores["ada"]}
	~ scores["bea"] = 12
	~ nested["ada"][2] = "done"

A write inserts the key when it is absent or replaces the existing value when
it is present. A read from a missing key is a runtime error. Key expressions
must match the declared key type: `Dict<string, V>` requires a string key and
`Dict<int, V>` requires an int key.

Dicts can be stored in globals, temps, constants, arrays, and struct fields,
and can be passed through function and external signatures:

	CONST BASE_SCORES: Dict<string, int> = %{"ada": 10}

	== function score_for(values: Dict<string, int>, key: string) => int ==
	~ return values[key]

	EXTERNAL load_scores(seed: int) => Dict<string, int>

Dict equality compares key type and entries recursively with `==` and `!=`.
Use Dict helpers to test, count, remove, and enumerate entries:

	{DICT_HAS(scores, "ada")}
	{DICT_SIZE(scores)}
	~ DICT_REMOVE(scores, "grace")
	~ temp keys: string[] = DICT_KEYS(scores)

`DICT_HAS(dict, key)` returns `true` when the key is present and `false`
otherwise. `DICT_SIZE(dict)` returns the current entry count.
`DICT_REMOVE(dict, key)` removes an existing entry, returns `void`, and does
nothing when the key is already absent. `DICT_KEYS(dict)` returns `string[]` for
`Dict<string, V>` and `int[]` for `Dict<int, V>`. String keys are returned in
lexical order; int keys are returned in ascending order.

### Enums

Named story states can be declared with `ENUM` at module top level. Enum
declarations list member names without commas, semicolons, or explicit member
values:

	ENUM State { Idle Busy Done }

	ENUM Mood {
		Calm
		Alarmed
	}

Enum names are nominal types. They can be used anywhere other maintained value
types can be used, including globals, constants, function parameters, function
returns, struct fields, and arrays:

	ENUM State { Idle Busy Done }

	STRUCT Actor {
		state: State
		history: State[]
	}

	VAR state: State
	VAR actor: Actor = {
		state: State.Busy,
		history: [State.Idle]
	}
	CONST DEFAULT_STATE: State = State.Done

An omitted enum initializer defaults to the first declared member, so the
`state` variable above starts as `State.Idle`. Enum members are referenced as
`State.Member` inside the same module. From another module, import the enum name
and use the qualified form:

	=== module game ===
	FROM data IMPORT State

	== main ==
	{data::State.Idle}
	-> DONE

Enum values can be assigned, passed through functions, printed in text output,
stored inside structs and arrays, compared with `==` and `!=`, and used in
`switch` case values:

	{ switch state:
	- State.Idle:
		Still waiting.
	- State.Busy:
		In motion.
	- else:
		Done.
	}

Enum values do not interoperate with strings. Even though compiled enum values
are stored as strings in the runtime format, source code must use enum members,
not string literals. Ordering and arithmetic operators are not defined for
enums.

### Using Global Variables

We can test global variables to control options, and provide conditional text, in a similar way to what we have previously seen.

	== the_train ==
		The train jolted and rattled. { mood > 0:I was feeling positive enough, however, and did not mind the odd bump|It was more than I could bear}.
		*	{ not knows_about_wager } 'But, Monsieur, why are we travelling?' I asked.
		* 	{ knows_about_wager} I contemplated our strange adventure. Would it be possible?

#### Advanced: divert target variables

Divert targets are typed values in ink-rs. They can be stored in globals, temps, constants, function parameters, function returns, struct fields, and arrays. A bare `->` declaration has no default value, so provide an initializer unless the type is `->[]`.

Static diverts use a literal path: `-> retreat`. Dynamic diverts use braces around an expression typed as `->`: `-> {next}`, `-> {route.next}`, `-> {targets[0]}`, or `-> {pick(flag)}`.

	== continue_or_quit ==
	Give up now, or keep trying to save your Kingdom?
	~ temp next: -> = -> more_hopeless_introspection
	*  Keep trying! 	-> {next}
	*  Give up 		-> retreat


#### Advanced: Global variables are externally visible

Global variables can be accessed, and altered, from the runtime as well from the story, so provide a good way to communicate between the wider game and the story.

The **ink** layer is often be a good place to store gameplay-variables; there's no save/load issues to consider, and the story itself can react to the current values.



### Printing variables

The value of a variable can be printed as content using the same braced inline syntax used by conditional text:

	VAR friendly_name_of_player: string = "Jackie"
	VAR age: int = 23

	My name is Jean Passepartout, but my friends call me {friendly_name_of_player}. I'm {age} years old.

This can be useful in debugging. For more complex printing based on logic and variables, see the section on [functions](#5-functions).

### Evaluating strings

It might be noticed that above we refered to variables as being able to contain "content", rather than "strings". That was deliberate, because a string defined in ink can contain ink - although it will always evaluate to a string. (Yikes!)

	VAR a_colour: string = ""

	~ a_colour = "red"

	{a_colour}

... produces `red`.

Note that a string variable stores its current string value. Reusing the same
variable prints the same value unless the story assigns a new one, so the
following:

	The goon hits you, and sparks fly before you eyes, {a_colour} and {a_colour}.

... won't produce a very interesting effect. (If you really want this to work, use a text function to print the colour!)


## 2) Logic

Obviously, our global variables are not intended to be constants, so we need a syntax for altering them.

Since by default, any text in an **ink** script is printed out directly to the screen, we use a markup symbol to indicate that a line of content is intended meant to be doing some numerical work, we use the `~` mark.

The following statements all assign values to variables:


	== set_some_variables ==
		~ knows_about_wager = true
		~ x = (x * x) - (y * y) + c
		~ y = 2 * x * y

and the following will test conditions:

	{ x == 1.2 }
	{ x / 2 > 4 }
	{ y - 1 <= x * x }

### Mathematics

**ink** supports the four basic mathematical operations (`+`, `-`, `*` and `/`), as well as `%` (or `mod`), which returns the remainder after integer division. `*`, `/`, `mod`, and `%` share the same precedence and associate left-to-right, so `8 * 100 / 56` is evaluated as `(8 * 100) / 56`. There's also POW for to-the-power-of:

	{POW(3, 2)} is 9.
	{POW(16, 0.5)} is 4.


If more complex operations are required, one can write functions (using recursion if necessary), or call out to external, game-code functions (for anything more advanced).


#### RANDOM(min, max)

Ink can generate random integers if required using the RANDOM function. RANDOM is authored to be like a dice (yes, pendants, we said *a dice*), so the min and max values are both inclusive.

	~ temp dice_roll: int = RANDOM(1, 6)

	~ temp lazy_grading_for_test_paper: int = RANDOM(30, 75)

	~ temp number_of_heads_the_serpent_has: int = RANDOM(3, 8)

The random number generator can be seeded for testing purposes, see the section of Game Queries and Functions section above.

#### Advanced: numeric results follow operand types

Results of operations - in particular, for division - are typed based on the type of the input. So integer division returns integer, but floating point division returns floating point results. Declarations still need explicit types.

	~ temp x: int = 2 / 3
	~ temp y: int = 7 / 3
	~ temp z: float = 1.2 / 0.5

assigns `x` to be 0, `y` to be 2 and `z` to be 2.4.

#### Advanced: INT(), FLOOR() and FLOAT()

In cases where you don't want implicit types, or you want to round off a variable, you can cast it directly.

	{INT(3.2)} is 3.
	{FLOOR(4.8)} is 4.
	{INT(-4.8)} is -4.
	{FLOOR(-4.8)} is -5.

	{FLOAT(4)} is, um, still 4.



### String queries

Oddly for a text-engine, **ink** doesn't have much in the way of string-handling: it's assumed that any string conversion you need to do will be handled by the game code (and perhaps by external functions.) But we support three basic queries - equality, inequality, and substring (which we call ? for reasons that will become clear in a later chapter).

The following all return true:

	{ "Yes, please." == "Yes, please." }
	{ "No, thank you." != "Yes, please." }
	{ "Yes, please" ? "ease" }

Strings of type `string` can also be concatenated with `+`.

	VAR greeting: string = "Hello"
	VAR name: string = "Ada"
	{greeting + ", " + name + "!"}

### Array builtins

`LEN(array)` returns the current length of an array as an `int`.

	VAR clues: string[] = ["ticket", "cipher", "key"]

	{LEN(clues)} clues remain.

`ARRAY_REMOVE(array, index)` removes the item at a zero-based index and returns `void`. It can be used in a logic line, or inline when you want the mutation to happen while printing no value.

	~ ARRAY_REMOVE(clues, 1)
	{clues[0]} and {clues[1]} remain.

`ARRAY_PUSH(array, value)` appends a value to the end of an array and returns
`void`. `ARRAY_INSERT(array, index, value)` inserts before `index`; `index` may
equal `LEN(array)` to insert at the end. The inserted value must match the array
element type.

	~ ARRAY_PUSH(clues, "map")
	~ ARRAY_INSERT(clues, 0, "riddle")

Arrays and structs compare by value, so equality checks recurse through nested arrays and fields.

	VAR first_scores: int[] = [1, 2]
	VAR second_scores: int[] = [1, 2]
	{ if first_scores == second_scores:
		The scores match.
	}


## 3) Conditional blocks (if/else)

We've seen conditionals used to control options and story content; **ink** also provides an equivalent of the normal if/else-if/else structure.

### A simple 'if'

Multiline control blocks must name the control form after the opening brace.
For a simple if, use `{ if condition:`:

	{ if x > 0:
		~ y = x - 1
	}

Else conditions can be provided:

	{ if x > 0:
		~ y = x - 1
	- else:
		~ y = x + 1
	}

### Extended if/else if/else blocks

For else-if chains, start with `{ if:` and put every branch condition on a
`- condition:` branch:

	{ if:
		- x > 0:
			~ y = x - 1
		- else:
			~ y = x + 1
	}

And using this form we can include 'else-if' conditions:

	{ if:
		- x == 0:
			~ y = 0
		- x > 0:
			~ y = x - 1
		- else:
			~ y = x + 1
	}

(Note, as with everything else, the white-space is purely for readability and has no syntactic meaning.)

An if block that starts with an opening condition, such as `{ if x > 0:`, may
only add `- else:`. Use the `{ if:` form when you need else-if branches.

### Switch blocks

There is also a switch form. In this form, `x` is a selector value, not a
boolean condition. Each branch value is compared with the selector using `==`,
so the case values must be comparable with the selector's type:

	{ switch x:
	- 0: 	zero
	- 1: 	one
	- 2: 	two
	- else: lots
	}

Use `- else:` for fallback content. Content before the first case is not a
switch fallback.

Keywordless multiline forms such as `{ x > 0:` and `{ x:` are not control
blocks in ink-rs. Use `if` or `switch` explicitly.

### For blocks

Typed arrays and Dicts can be iterated with a multiline `for` control block.
This is source-level compiler syntax; it lowers to existing runtime variables,
conditionals, and collection helper calls.

Array loops can bind only the item, or both the zero-based index and item:

	{ for item in items:
		Item {item.name}
		~ total += item.score
	}

	{ for index, item in items:
		{index}: {item.name}
	}

Dict loops bind both key and value. Key-only Dict loops are not supported:

	{ for key, value in scores:
		{key}: {value}
	}

For `Dict<string, V>`, `key` is a `string`; for `Dict<int, V>`, `key` is an
`int`. Dict keys are iterated in the same stable order as `DICT_KEYS`: string
keys in lexical order and int keys in ascending order.

Loop variables are scoped to the loop body and can shadow outer source names.
They are visible inside nested `if`, `switch`, and nested `for` blocks, but not
after the loop.

For loop bodies may contain text lines with inline expressions, `~` logic
lines including typed `temp` declarations and assignments, simple `if` blocks,
extended `if:` blocks, `switch` blocks, and nested `for` blocks. The same
subset applies recursively inside nested branches and loops.

Choices, threads, diverts, tunnels, tunnel onwards, gathers, flow
declarations, `~ return`, global `VAR`, `CONST`, `ENUM`, `STRUCT`,
`EXTERNAL`, tags, and author warnings are not supported inside `for` blocks.
Use dynamic choices when an array should produce a runtime number of choices.
Use an ordinary knot/function helper when iteration needs arbitrary flow
control beyond the supported block subset.

Array loops fix `LEN(array)` once before the loop starts. Each iteration still
reads `array[index]`. Dict loops fix `DICT_KEYS(dict)` once before the loop
starts. Each iteration still reads `dict[key]`. If the body mutates the
iterated collection and a later fixed index or key is no longer readable, the
normal runtime array index or Dict missing-key error is preserved.

For flow checking, an `if`/`else` block or switch block closes the current flow
when every branch ends in a divert, `-> DONE`, `-> END`, choice, or return and
the block has an `else` branch. A bool switch that explicitly covers both
`true` and `false` is also exhaustive. Other switches are not assumed to cover
every value; add `- else:` or put an explicit `-> DONE`, `-> END`, choice, or
divert after the block.

#### Example: context-relevant content

These tests should use explicit variables. The following construction is a common way of saying "do some content which is relevant to the current game state":

	== dream ==
		{ if:
			- visited_snakes && not dream_about_snakes:
				~ fear++
				-> dream_about_snakes

			- visited_poland && not dream_about_polish_beer:
				~ fear--
				-> dream_about_polish_beer

			- else:
				// breakfast-based dreams have no effect
				-> dream_about_marmalade
		}

The syntax has the advantage of being easy to extend, and prioritise.



### Conditional blocks are not limited to logic

Conditional blocks can be used to control story content as well as logic:

	I stared at Monsieur Fogg.
	{ if know_about_wager:
		<> "But surely you are not serious?" I demanded.
	- else:
		<> "But there must be a reason for this trip," I observed.
	}
	He said nothing in reply, merely considering his newspaper with as much thoroughness as entomologist considering his latest pinned addition.

You can even put options inside conditional blocks:

	{ if door_open:
		* 	I strode out of the compartment and I fancied I heard my master quietly tutting to himself. 			-> go_outside
	- else:
		*	I asked permission to leave and Monsieur Fogg looked surprised. 	-> open_door
		* 	I stood and went to open the door. Monsieur Fogg seemed untroubled by this small rebellion. -> open_door
	}

...but note that the lack of weave-syntax and nesting in the above example isn't accidental: to avoid confusing the various kinds of nesting at work, you aren't allowed to include gather points inside conditional blocks.

### Multiline blocks

Multiline conditionals are the supported multiline brace block form in ink-rs.
They must start with an explicit `{ if ...:`, `{ if:`, or `{ switch ...:`
header.

	VAR luck_step: int = 0

	{ if:
	- luck_step == 0:
		Would my luck hold?
		~ luck_step = 1
	- else:
		Could I win the hand?
	}


## 4) Temporary Variables

### Temporary variables are for scratch calculations

Sometimes, a global variable is unwieldy. **ink** provides temporary variables for quick calculations of things.

	== near_north_pole ==
		~ temp number_of_warm_things: int = 0
		{ if blanket:
			~ number_of_warm_things++
		}
		{ if ear_muffs:
			~ number_of_warm_things++
		}
		{ if gloves:
			~ number_of_warm_things++
		}
		{ if number_of_warm_things > 2:
			Despite the snow, I felt incorrigibly snug.
		- else:
			That night I was colder than I have ever been.
		}

The value in a temporary variable is thrown away after the story leaves the stitch in which it was defined.

### Knots and stitches can take parameters

A particularly useful form of temporary variable is a parameter. Any knot or stitch can be given a value as a parameter.

	*	Accuse Hasting
			-> accuse("Hastings")
	*	Accuse Mrs Black
			-> accuse("Claudia")
	*	Accuse myself
			-> accuse("myself")

	== accuse(who) ==
		"I accuse {who}!" Poirot declared.
		"Really?" Japp replied. "{who == "myself":You did it?|{who}?}"
		"And why not?" Poirot shot back.


... and you'll need to use parameters if you want to pass a temporary value from one stitch to another!

#### Example: a recursive knot definition

Temporary variables are safe to use in recursion (unlike globals), so the following will work.

	-> add_one_to_one_hundred(0, 1)

	== add_one_to_one_hundred(total, x) ==
		~ total = total + x
		{ if x == 100:
			-> finished(total)
		- else:
			-> add_one_to_one_hundred(total, x + 1)
		}

	== finished(total) ==
		"The result is {total}!" you announce.
		Gauss stares at you in horror.
		-> END


(In fact, this kind of definition is useful enough that **ink** provides a special kind of knot, called, imaginatively enough, a `function`, which comes with certain restrictions and can return a value. See the section below.)


#### Advanced: sending divert targets as parameters

Knot/stitch addresses can be passed to knot and stitch parameters as divert targets. Declare the parameter type as `name: ->`, and construct target values with `-> target`:

	== sleeping_in_hut ==
		You lie down and close your eyes.
		-> generic_sleep (-> waking_in_the_hut)

	== generic_sleep(waking: ->) ==
		You sleep perchance to dream etc. etc.
		-> {waking}

	== waking_in_the_hut ==
		You get back to your feet, ready to continue your journey.

The `->` argument in the call constructs a divert target value. Passing a bare target name as a value is not the same thing as a target value and will not satisfy a `->` parameter:

	== sleeping_in_hut ==
		You lie down and close your eyes.
		-> generic_sleep (waking_in_the_hut)

Use `-> waking_in_the_hut` when constructing the value, and `-> {waking}` when diverting through the parameter.





## 5) Functions

The use of parameters on knots means they are almost functions in the usual sense, but they lack one key concept - that of the call stack, and the use of return values.

**ink** includes functions: they are knots, with the following limitations and features:

A function:
- cannot contain stitches
- cannot use diverts or offer choices
- can call other functions
- can include printed content
- can return a value of any declared type
- can recurse safely

(Some of these may seem quite limiting, but for more story-oriented call-stack-style features, see the section on [Tunnels](#1-tunnels).)

Function parameters must be declared with explicit types, and every function must declare a return type after `=>`. Return values are provided via the `~ return` statement. Use `=> void` when a function performs an effect and does not return a value. Use `=> ->` when a function returns a divert target value.

### Defining and calling functions

To define a function, simply declare a knot to be one:

	== function say_yes_to_everything() => bool ==
		~ return true

	== function lerp(a: float, b: float, k: float) => float ==
		~ return ((b - a) * k) + a

Functions are called by name, and with brackets, even if they have no parameters:

	~ temp x: float = lerp(2.0, 8.0, 0.3)

	*	{say_yes_to_everything()} 'Yes.'

As in any other language, a function, once done, returns the flow to wherever it was called from - and despite not being allowed to divert the flow, functions can still call other functions.

	== function say_no_to_nothing() => bool ==
		~ return say_yes_to_everything()

### Functions don't have to return anything

A function does not need to have a return value, and can simply do something that is worth packaging up:

	== function harm(x: int) => void ==
		{ if stamina < x:
			~ stamina = 0
		- else:
			~ stamina = stamina - x
		}

...though remember a function cannot divert, so while the above prevents a negative Stamina value, it won't kill a player who hits zero.

### Functions can be called inline

Functions can be called on `~` content lines, but can also be called during a piece of content. In this context, the return value, if there is one, is printed (as well as anything else the function wants to print.) If there is no return value, nothing is printed.

Content is, by default, 'glued in', so the following:

	Monsieur Fogg was looking {describe_health(health)}.

	== function describe_health(x: int) => string ==
	{ if:
	- x == 100:
		~ return "spritely"
	- x > 75:
		~ return "chipper"
	- x > 45:
		~ return "somewhat flagging"
	- else:
		~ return "despondent"
	}

produces:

	Monsieur Fogg was looking despondent.

#### Examples

For instance, you might include:

	== function max(a: int, b: int) => int ==
		{ if a < b:
			~ return b
		- else:
			~ return a
		}

	== function exp(x: int, e: int) => int ==
		// returns x to the power e where e is an integer
		{ if e <= 0:
			~ return 1
		- else:
			~ return x * exp(x, e - 1)
		}

Then:

	The maximum of 2^5 and 3^3 is {max(exp(2,5), exp(3,3))}.

produces:

	The maximum of 2^5 and 3^3 is 32.


#### Example: turning numbers into words

The following example is long, but appears in pretty much every inkle game to date. It uses explicit `{ if:` blocks for condition branches and `{ switch ...:` blocks when comparing one selector value against multiple cases.

    == function print_num(x: int) => void ==
    { if:
        - x >= 1000:
            {print_num(x / 1000)} thousand { x mod 1000 > 0:{print_num(x mod 1000)}}
        - x >= 100:
            {print_num(x / 100)} hundred { x mod 100 > 0:and {print_num(x mod 100)}}
        - x == 0:
            zero
        - else:
            { if x >= 20:
                { switch x / 10:
                    - 2: twenty
                    - 3: thirty
                    - 4: forty
                    - 5: fifty
                    - 6: sixty
                    - 7: seventy
                    - 8: eighty
                    - 9: ninety
                }
                { x mod 10 > 0:<>-<>}
            }
            { if x < 10 || x > 20:
                { switch x mod 10:
                    - 1: one
                    - 2: two
                    - 3: three
                    - 4: four
                    - 5: five
                    - 6: six
                    - 7: seven
                    - 8: eight
                    - 9: nine
                }
            - else:
                { switch x:
                    - 10: ten
                    - 11: eleven
                    - 12: twelve
                    - 13: thirteen
                    - 14: fourteen
                    - 15: fifteen
                    - 16: sixteen
                    - 17: seventeen
                    - 18: eighteen
                    - 19: nineteen
                }
            }
    }

which enables us to write things like:

	~ temp price: int = 15

	I pulled out {print_num(price)} coins from my pocket and slowly counted them.
	"Oh, never mind," the trader replied. "I'll take half." And she took {print_num(price / 2)}, and pushed the rest back over to me.



### Parameters can be passed by reference

Function parameters can also be passed 'by reference', meaning that the function can actually alter the the variable being passed in, instead of creating a temporary variable with that value.

For instance, most **inkle** stories include the following:

	== function alter(ref x: int, k: int) => void ==
		~ x = x + k

Lines such as:

	~ gold = gold + 7
	~ health = health - 4

then become:

	~ alter(gold, 7)
	~ alter(health, -4)

which are slightly easier to read, and (more usefully) can be done inline for maximum compactness.

	*	I ate a biscuit and felt refreshed. {alter(health, 2)}
	* 	I gave a biscuit to Monsieur Fogg and he wolfed it down most undecorously. {alter(foggs_health, 1)}
	-	<> Then we continued on our way.

Wrapping up simple operations in function can also provide a simple place to put debugging information, if required.




##  6) Constants


### Global Constants

Interactive stories often rely on state machines, tracking what stage some higher level process has reached. There are lots of ways to do this, but the most conveninent is to use constants.

In ink-rs, every module-level constant declaration must include an explicit type using `CONST name: Type = value`. Constants can use the same maintained value types as variables, including structs, arrays, and Dicts such as `Player`, `Player[]`, and `Dict<string, int>`.

Sometimes, it's convenient to define constants to be strings, so you can print them out, for gameplay or debugging purposes.

	CONST HASTINGS: string = "Hastings"
	CONST POIROT: string = "Poirot"
	CONST JAPP: string = "Japp"

	VAR current_chief_suspect: string = HASTINGS

	== review_evidence ==
		{ if found_japps_bloodied_glove:
			~ current_chief_suspect = POIROT
		}
		Current Suspect: {current_chief_suspect}

Sometimes giving them values is useful:

	CONST PI: float = 3.14
	CONST VALUE_OF_TEN_POUND_NOTE: int = 10

And sometimes the numbers are useful in other ways:

	CONST LOBBY: int = 1
	CONST STAIRCASE: int = 2
	CONST HALLWAY: int = 3

	CONST HELD_BY_AGENT: int = -1

	VAR secret_agent_location: int = LOBBY
	VAR suitcase_location: int = HALLWAY

	== report_progress ==
	{ if:
        -  secret_agent_location == suitcase_location:
		The secret agent grabs the suitcase!
		~ suitcase_location = HELD_BY_AGENT

	-  secret_agent_location < suitcase_location:
		The secret agent moves forward.
		~ secret_agent_location++
	}

Constants are simply a way to allow you to give story states easy-to-understand names.

## 7) Advanced: Game-side logic

External function declarations in ink allow you to directly call host functions in the game.

In ink-rs, every `EXTERNAL` declaration is module-level and needs a typed signature:

	=== module audio ===
	STRUCT Player {
		hp: int
	}

	EXTERNAL next_score(value: int) => int
	EXTERNAL make_scores() => int[]
	EXTERNAL load_scores(seed: int) => Dict<string, int>
	EXTERNAL make_player() => Player
	EXTERNAL next_scene(name: string) => ->
	EXTERNAL log_event(message: string) => void

External calls are type-checked like Ink function calls. Host bindings use the
module-qualified source name, such as `audio::next_score`. Host return values
must match the declared runtime shape: primitive values for primitive returns,
arrays for `T[]`, Dict values with the declared key type for `Dict<K, V>`,
objects with matching fields for struct returns, and divert target values for
`->` returns.

`INTERNAL` declarations expose ink functions for host code to call through the
runtime API:

	=== module config ===
	VAR reads: int = 0

	== INTERNAL read_config(key: string) => string ==
	~ reads = reads + 1
	~ return key

Host calls use the source-qualified name:

	story.call_internal("config::read_config", Some(&args))

`INTERNAL` functions are implemented in ink, may modify story state, and should
return host-callable values with `~ return`. Ordinary `function` declarations
remain ink-only helpers and are not exposed through `call_internal`.

Modules that declare `INTERNAL` functions are compiled as host-callable roots
even when they are not imported by the `main` module. Their import dependencies
are compiled with them. Other modules that are unreachable from `main` or an
`INTERNAL` module are omitted from compiled JSON.

# Part 4: Advanced Flow Control


## 1) Tunnels

The default structure for **ink** stories is a "flat" tree of choices, branching and joining back together, perhaps looping, but with the story always being "at a certain place".

But this flat structure makes certain things difficult: for example, imagine a game in which the following interaction can happen:

	== crossing_the_date_line ==
	*	"Monsieur!" I declared with sudden horror. "I have just realised. We have crossed the international date line!"
	-	Monsieur Fogg barely lifted an eyebrow. "I have adjusted for it."
	*	I mopped the sweat from my brow. A relief!
	* 	I nodded, becalmed. Of course he had!
	*  I cursed, under my breath. Once again, I had been belittled!

...but it can happen at several different places in the story. We don't want to have to write copies of the content for each different place, but when the content is finished it needs to know where to return to. We can do this using parameters:

	== crossing_the_date_line(return_to: ->) ==
	...
	-	-> {return_to}

	...

	== outside_honolulu ==
	We arrived at the large island of Honolulu.
	- (postscript)
		-> crossing_the_date_line(-> done)
	- (done)
		-> END

	...

	== outside_pitcairn_island ==
	The boat sailed along the water towards the tiny island.
	- (postscript)
		-> crossing_the_date_line(-> done)
	- (done)
		-> END

Both of these locations now call and execute the same segment of storyflow, but once finished they return to where they need to go next.

But what if the section of story being called is more complex - what if it spreads across several knots? Using the above, we'd have to keep passing the 'return-to' parameter from knot to knot, to ensure we always knew where to return.

So instead, **ink** integrates this into the language with a new kind of divert, that functions rather like a subroutine, and is called a 'tunnel'.

### Tunnels run sub-stories

The tunnel syntax looks like a divert, with another divert on the end:

	-> crossing_the_date_line ->

This means "do the crossing_the_date_line story, then continue from here".

Inside the tunnel itself, the syntax is simplified from the parameterised example: all we do is end the tunnel using the `->->` statement which means, essentially, "go on".

	== crossing_the_date_line ==
	// this is a tunnel!
	...
	- 	->->

Note that tunnel knots aren't declared as such, so the compiler won't check that tunnels really do end in `->->` statements, except at run-time. So you will need to write carefully to ensure that all the flows into a tunnel really do come out again.

Tunnels can also be chained together, or finish on a normal divert:

	...
	// this runs the tunnel, then diverts to 'done'
	-> crossing_the_date_line -> done
	...

	...
	//this runs one tunnel, then another, then diverts to 'done'
	-> crossing_the_date_line -> check_foggs_health -> done
	...

Tunnels can be nested, so the following is valid:

	== plains ==
	= night_time
		The dark grass is soft under your feet.
		*	Sleep
			-> sleep_here -> wake_here -> day_time
	= day_time
		It is time to move on.

	== wake_here ==
		You wake as the sun rises.
		*	Eat something
			-> eat_something ->
		*	Make a move
		-	->->

	== sleep_here ==
		You lie down and try to close your eyes.
		-> monster_attacks ->
		Then it is time to sleep.
		-> dream ->
		->->

... and so on.


#### Advanced: Tunnels can return elsewhere

Sometimes, in a story, things happen. So sometimes a tunnel can't guarantee that it will always want to go back to where it came from. **ink** supplies a syntax to allow you to "returning from a tunnel but actually go somewhere else" but it should be used with caution as the possibility of getting very confused is very high indeed.

Still, there are cases where it's indispensable:

	== fall_down_cliff ==
	-> hurt(5) ->
	You're still alive! You pick yourself up and walk on.

	== hurt(x) ==
		~ stamina -= x
		{ if stamina <= 0:
			->-> youre_dead
		}

	== youre_dead ==
	Suddenly, there is a white light all around you. Fingers lift an eyepiece from your forehead. 'You lost, buddy. Out of the chair.'

And even in less drastic situations, we might want to break up the structure:

	-> talk_to_jim ->

	== talk_to_jim ==
	 - (opts)
		*	 Ask about the warp lacelles
			-> warp_lacells ->
		*	 Ask about the shield generators
			-> shield_generators ->
		* 	 Stop talking
			->->
	 - -> opts

	 = warp_lacells
		{ shield_generators : ->-> argue }
		"Don't worry about the warp lacelles. They're fine."
		->->

	 = shield_generators
		{ warp_lacells : ->-> argue }
		"Forget about the shield generators. They're good."
		->->

	 = argue
		"What's with all these questions?" Jim demands, suddenly.
		...
		->->

#### Advanced: Tunnels use a call-stack

Tunnels are on a call-stack, so can safely recurse.


## 2) Threads

So far, everything in ink has been entirely linear, despite all the branching and diverting. But it's actually possible for a writer to 'fork' a story into different sub-sections, to cover more possible player actions.

We call this 'threading', though it's not really threading in the sense that computer scientists mean it: it's more like stitching in new content from various places.

Note that this is definitely an advanced feature: the engineering stories becomes somewhat more complex once threads are involved!

### Threads join multiple sections together

Threads allow you to compose sections of content from multiple sources in one go. For example:

    === module game ===

    == main ==
    I had a headache; threading is hard to get your head around.
    <- conversation
    <- walking


    == conversation ==
    It was a tense moment for Monty and me.
     * "What did you have for lunch today?" I asked.
        "Spam and eggs," he replied.
     * "Nice weather, we're having," I said.
        "I've seen better," he replied.
     - -> house

    == walking ==
    We continued to walk down the dusty road.
     * Continue walking
        -> house

    == house ==
    Before long, we arrived at his house.
    -> END

It allows multiple sections of story to combined together into a single section:

    I had a headache; threading is hard to get your head around.
    It was a tense moment for Monty and me.
    We continued to walk down the dusty road.
    1: "What did you have for lunch today?"
    2: "Nice weather, we're having,"
    3: Continue walking

On encountering a thread statement such as `<- conversation`, the compiler will fork the story flow. The first fork considered will run the content at `conversation`, collecting up any options it finds. Once it has run out of flow here it'll then run the other fork.

All the content is collected and shown to the player. But when a choice is chosen, the engine will move to that fork of the story and collapse and discard the others.

Note that global variables are *not* forked. This is useful when separate threads should still share authored state.

### Uses of threads

In a normal story, threads might never be needed.

But for games with lots of independent moving parts, threads quickly become essential. Imagine a game in which characters move independently around a map: the main story hub for a room might look like the following:

	=== module game ===

	CONST HALLWAY: int = 1
	CONST OFFICE: int = 2

	VAR player_location: int = HALLWAY
	VAR generals_location: int = HALLWAY
	VAR doctors_location: int = OFFICE

	== main ==
	-> run_player_location

	== run_player_location ==
		{ if:
			- player_location == HALLWAY: -> hallway
		}

	== hallway ==
		<- characters_present(HALLWAY)
		*	Drawers	-> examine_drawers
		* 	Wardrobe -> examine_wardrobe
		*  Go to Office 	-> go_office
		-	-> run_player_location
	= examine_drawers
		// etc...

	// Here's the thread, which mixes in dialogue for characters you share the room with at the moment.

	== characters_present(room) ==
		{ if generals_location == room:
			<- general_conversation
		}
		{ if doctors_location == room:
			<- doctor_conversation
		}
		-> DONE

	== general_conversation ==
		*	Ask the General about the bloodied knife
			"It's a bad business, I can tell you."
		-	-> run_player_location

	== doctor_conversation ==
		*	Ask the Doctor about the bloodied knife
			"There's nothing strange about blood, is there?"
		-	-> run_player_location



Note in particular, that we need an explicit way to return the player who has gone down a side-thread to return to the main flow. In most cases, threads will either need a parameter telling them where to return to, or they'll need to end the current story section.


### When does a side-thread end?

Side-threads end when they run out of flow to process: and note, they collect up options to display later (unlike tunnels, which collect options, display them and follow them until they hit an explicit return, possibly several moves later).

Sometimes a thread has no content to offer - perhaps there is no conversation to have with a character after all, or perhaps we have simply not written it yet. In that case, we must mark the end of the thread explicitly.

If we didn't, the end of content might be a story-bug or a hanging story thread, and we want the compiler to tell us about those.

### Using `-> DONE`

In cases where we want to mark the end of a thread, we use `-> DONE`: meaning "the flow intentionally ends here". If we don't, we might end up with a warning message - we can still play the game, but it's a reminder that we have unfinished business.

The example at the start of this section will generate a warning; it can be fixed as follows:

    == main ==
    I had a headache; threading is hard to get your head around.
    <- conversation
    <- walking
    -> DONE

The extra DONE tells ink that the flow here has ended and it should rely on the threads for the next part of the story.

Note that we don't need a `-> DONE` if the flow ends with options that fail their conditions. The engine treats this as a valid, intentional, end of flow state.

**You do not need a `-> DONE` after an option has been chosen**. Once an option is chosen, the thread rejoins normal story flow.

Using `-> END` in this case will not end the thread, but the whole story flow. (And this is the real reason for having two different ways to end flow.)


#### Example: adding the same choice to several places

Threads can be used to add the same choice into lots of different places. When using them this way, it's normal to pass a divert as a parameter, to tell the story where to go after the choice is done.

	VAR reviewed_notes_recently: bool = false

	== outside_the_house ==
	The front step. The house smells. Of murder. And lavender.
	- (top)
		<- review_case_notes(-> top)
		*	Go through the front door
			I stepped inside the house.
			-> the_hallway
		* 	Sniff the air
			I hate lavender. It makes me think of soap, and soap makes me think about my marriage.
			-> top

	== the_hallway ==
	The hallway. Front door open to the street. Little bureau.
	- (top)
		<- review_case_notes(-> top)
		*	Go through the front door
			I stepped out into the cool sunshine.
			-> outside_the_house
		* 	Open the bureau
			Keys. More keys. Even more keys. How many locks do these people need?
			-> top

	== review_case_notes(go_back_to: ->) ==
	*	{not reviewed_notes_recently}
		Review my case notes
		// the explicit variable controls whether this option repeats immediately
		~ reviewed_notes_recently = true
		I flicked through the notes I'd made so far. Still not obvious suspects.
	- 	(done) -> {go_back_to}

Note this is different than a tunnel, which runs the same block of content but doesn't give a player a choice. So a layout like:

	<- childhood_memories(-> next)
	*	Look out of the window
		I daydreamed as we rolled along...
	 - (next) Then the whistle blew...

might do exactly the same thing as:

	*	Remember my childhood
		-> think_back ->
	*	Look out of the window
		I daydreamed as we rolled along...
	- 	(next) Then the whistle blew...

but as soon as the option being threaded in includes multiple choices, or conditional logic on choices (or any text content, of course!), the thread version becomes more practical.


#### Example: organisation of wide choice points

A game which uses ink as a script rather than a literal output might often generate very large numbers of parallel choices, intended to be filtered by the player via some other in-game interaction - such as walking around an environment. Threads can be useful in these cases simply to divide up choices.

```
=== module game ===

== main ==
-> the_kitchen

== the_kitchen ==
- (top)
	<- drawers(-> top)
	<- cupboards(-> top)
	<- room_exits
= drawers(goback: ->)
	// choices about the drawers...
	...
= cupboards(goback: ->)
	// choices about cupboards
	...
= room_exits
	// exits; doesn't need a "return point" as if you leave, you go elsewhere
	...
```

#### Example: generating choices from an array

Use dynamic choices when an array should produce a runtime number of choices.
The dynamic binding goes immediately after the choice marker and the generated
choices behave like ordinary choices.

```
=== module game ===

STRUCT ChoiceOption {
	text: string
	enabled: bool
}

VAR options: ChoiceOption[] = [
	%ChoiceOption{
		text: "A",
		enabled: true
	},
	%ChoiceOption{
		text: "B",
		enabled: true
	},
	%ChoiceOption{
		text: "C",
		enabled: false
	},
	%ChoiceOption{
		text: "D",
		enabled: true
	}
]

== main ==
* [i, option in options] {option.enabled}: {option.text}
	Choice {i}: {option.text}.
	-> DONE
* ->
	No options remain.
	-> DONE
```

This presents `A`, `B`, and `D`; `C` is skipped because its `enabled` field is
false.

The array expression is evaluated once before expansion. Each generated choice
captures its own `i` and `option` values in the normal choice thread, so pending
choices can be saved and loaded without regenerating the list.

The colon after `{option.enabled}` is deliberate. At the beginning of a choice
line, braced expressions are parsed as choice conditions. The colon ends the
condition prefix, so the following `{option.text}` is parsed as dynamic choice
text instead of another condition.

# Part 5: International character support in identifiers

By default, ink has no limitations on the use of non-ASCII characters inside the story content. However, a limitation currently exsits
on the characters that can be used for names of constants, variables, stictches, diverts and other named flow elements (a.k.a. *identifiers*).

Sometimes it is inconvenient for a writer using a non-ASCII language to write a story because they have to constantly switch to naming identifiers in ASCII and then switching back to whatever language they are using for the story. In addition, naming identifiers in the author's own language could improve the overal readibility of the raw story format.

In an effort to assist in the above scenario, ink *automatically* supports a set of pre-defined non-ASCII character ranges that can be used as identifiers. In general, those ranges have been selected to include the alpha-numeric subset of the official unicode character range, which would suffice for naming identifiers. The below section gives more detailed information on the non-ASCII characters that ink automatically supports.

### Supported Identifier Characters

The support for the additional character ranges in ink is currently limited to a predefined set of character ranges.

Below are the currently supported identifier ranges.

 - **Arabic**

   Enables characters for languages of the Arabic family and is a subset of the official *Arabic* unicode range `\u0600`-`\u06FF`.


 - **Armenian**

   Enables characters for the Armenian language and is a subset of the official *Armenian* unicode range `\u0530`-`\u058F`.


 - **Cyrillic**

   Enables characters for languages using the Cyrillic alphabet and is a subset of the official *Cyrillic* unicode range `\u0400`-`\u04FF`.


 - **Greek**

   Enables characters for languages using the Greek alphabet and is a subset of the official *Greek and Coptic* unicode range `\u0370`-`\u03FF`.


 - **Hebrew**

   Enables characters in Hebrew using the Hebrew alphabet and is a subset of the official *Hebrew* unicode range `\u0590`-`\u05FF`.


 - **Latin Extended A**

   Enables an extended character range subset of the Latin alphabet - completely represented by the official *Latin Extended-A* unicode range `\u0100`-`\u017F`.


 - **Latin Extended B**

   Enables an extended character range subset of the Latin alphabet - completely represented by the official *Latin Extended-B* unicode range `\u0180`-`\u024F`.

- **Latin 1 Supplement**

   Enables an extended character range subset of the Latin alphabet - completely represented by the official *Latin 1 Supplement* unicode range `\u0080` - `\u00FF`.


**NOTE!** ink files should be saved in UTF-8 format, which ensures that the above character ranges are supported.

For new identifier character range requests, open an issue or pull request in this project.
