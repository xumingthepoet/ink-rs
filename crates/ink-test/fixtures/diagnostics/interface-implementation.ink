=== interface IItem ===
== target(amount: int) ==
== function score(amount: int) => int ==

=== module good implements IItem ===
== main ==
-> END
== target(amount: int) ==
-> END
== function score(amount: int) => int ==
~ return amount

=== module unknown implements IMissing ===
== helper ==
-> END

=== module missing implements IItem ===
== helper ==
-> END

=== module wrongKind implements IItem ===
== function target(amount: int) => void ==
~ return
== function score(amount: int) => int ==
~ return amount

=== module wrongSignature implements IItem ===
== target(amount: string) ==
-> END
== function score(amount: string) => string ==
~ return amount

=== module externalImpl implements IItem ===
EXTERNAL score(amount: int) => int
== target(amount: int) ==
-> END
