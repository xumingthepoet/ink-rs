=== module game ===
ENUM RoomKind { Key Chest }

STRUCT RoomDef {
kind: RoomKind
amount: int
}

CONST rooms: Dict<int, RoomDef> = %{
    1: %RoomDef{ kind: RoomKind.Key, amount: 7 },
    2: %RoomDef{ kind: RoomKind.Chest, amount: 3 }
}

== main ==
{ if DICT_HAS(rooms, 1):
    ~ temp room: RoomDef = rooms[1]
    {room.kind}|{room.amount}
- else:
    missing-one
}
{ if DICT_HAS(rooms, 9):
    unexpected
- else:
    missing-nine
}
