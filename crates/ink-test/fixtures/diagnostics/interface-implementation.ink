=== interface IItem ===
== target(amount: int) ==
== function score(amount: int) => int ==

=== module good implements IItem ===
== main ==
== target(amount: int) ==
== function score(amount: int) => int ==
~ return amount

=== module unknown implements IMissing ===
== helper ==

=== module missing implements IItem ===
== helper ==

=== module wrongKind implements IItem ===
== function target(amount: int) => void ==
~ return
== function score(amount: int) => int ==
~ return amount

=== module wrongSignature implements IItem ===
== target(amount: string) ==
== function score(amount: string) => string ==
~ return amount

=== module externalImpl implements IItem ===
EXTERNAL score(amount: int) => int
== target(amount: int) ==
