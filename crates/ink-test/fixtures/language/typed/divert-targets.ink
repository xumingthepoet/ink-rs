VAR next: -> = -> here
CONST fallback: -> = -> fallback_target
VAR targets: ->[] = [-> array_target]

STRUCT Route {
next: ->
}

VAR route: Route = { next: -> struct_target }

-> {next}

== here
Here.
-> {pick_struct()}

== function pick_struct() => -> ==
~ return route.next

== struct_target
Struct.
-> {targets[0]}

== array_target
Array.
-> {pick_fallback()}

== function pick_fallback() => -> ==
~ return fallback

== fallback_target
Fallback.
-> END
