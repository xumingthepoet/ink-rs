=== module game ===
VAR next: -> = -> first
CONST fallback: -> = -> const_target
CONST const_targets: ->[] = [-> const_array_target]
VAR targets: ->[] = [-> array_target]
STRUCT Route {
next: ->
}
VAR route: Route = %Route{ next: -> struct_target }
CONST const_route: Route = %Route{ next: -> const_struct_target }

== main ==
-> {next}

== first ==
First.
-> {route.next}

== struct_target ==
Struct.
-> {targets[0]}

== array_target ==
Array.
-> {fallback}

== const_target ==
Const.
-> {const_route.next}

== const_struct_target ==
Const struct.
-> {const_targets[0]}

== const_array_target ==
Const array.
-> {pick(true)}

== function pick(flag: bool) => -> ==
{ if flag:
    ~ return -> final
- else:
    ~ return -> first
}

== final ==
Final.
-> END
