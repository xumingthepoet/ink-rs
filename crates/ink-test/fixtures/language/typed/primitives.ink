VAR score: int
VAR ratio: float = 1.5
VAR ready: bool = true
VAR name: string = "Ada"

~ temp local: int = score + 2
~ name = name + " Lovelace"
{score}|{ratio}|{ready}|{name}|{local}|{score == 0}
-> DONE
