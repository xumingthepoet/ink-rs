=== module game ===
VAR width: int = 3

== main ==
{border_row(0, "")}
{literal_tokens()}
{interpolated("ok")}

== function border_row(x: int, text: string) => string ==
{ if x >= width + 2:
    ~ return text
- else:
    ~ return border_row(x + 1, text + "#")
}

== function literal_tokens() => string ==
~ return "# <> -> <-"

== function interpolated(value: string) => string ==
~ return "value {value} # <> -> <-"
