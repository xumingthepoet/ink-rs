=== module game ===

STRUCT Exhibit {
    id: int
    name: string
    theme: string
    quality: int
    appeal: int
}

VAR exhibit_catalog: Exhibit[] = [
    %Exhibit{id: 101, name: "Marble Procession", theme: "Renaissance", quality: 88, appeal: 74},
    %Exhibit{id: 102, name: "Copper Street Lamp", theme: "Modern", quality: 62, appeal: 55},
    %Exhibit{id: 103, name: "Clockwork Cathedral", theme: "Modern", quality: 90, appeal: 82},
    %Exhibit{id: 104, name: "Glass Lantern Archive", theme: "Scientific", quality: 70, appeal: 63},
    %Exhibit{id: 105, name: "Botanical Engine", theme: "Scientific", quality: 83, appeal: 76},
    %Exhibit{id: 106, name: "Marble Altar", theme: "Renaissance", quality: 86, appeal: 80},
    %Exhibit{id: 107, name: "Telescope Dome", theme: "Scientific", quality: 77, appeal: 61},
    %Exhibit{id: 108, name: "City of Windows", theme: "Modern", quality: 79, appeal: 71}
]

VAR exhibit_index: Dict<int, int> = %{
    101: 0,
    102: 1,
    103: 2,
    104: 3,
    105: 4,
    106: 5,
    107: 6,
    108: 7
}

VAR selected_ids: int[] = [101, 103, 105, 106]

VAR selected_count: int = 0
VAR total_quality: int = 0
VAR total_appeal: int = 0
VAR renaissance_count: int = 0
VAR modern_count: int = 0
VAR scientific_count: int = 0
VAR other_count: int = 0
VAR thematic_coherence: int = 0
VAR prestige_score: int = 0
VAR visitor_response: string = ""

== main ==
Curator planning starts the weekly gallery briefing.
~ print_exhibit_catalog(0)
~ print_selected_gallery()
~ evaluate_lineup()
~ print_curation_report()
-> DONE

== function print_exhibit_catalog(index: int) => void ==
{ if index >= LEN(exhibit_catalog):
    Catalog review complete.
- else:
    ~ temp exhibit: Exhibit = exhibit_catalog[index]
    Exhibit {exhibit.id}: {exhibit.name}
    Theme {exhibit.theme} | quality {exhibit.quality} | appeal {exhibit.appeal}
    { if is_selected_id(exhibit.id, 0):
        Marked for this floor.
    - else:
        Set aside for reserve.
    }
    ~ print_exhibit_catalog(index + 1)
}

== function print_selected_gallery() => void ==
The curated floor selection includes:

== function evaluate_lineup() => void ==
~ selected_count = LEN(selected_ids)
~ total_quality = 0
~ total_appeal = 0
~ renaissance_count = 0
~ modern_count = 0
~ scientific_count = 0
~ other_count = 0
~ evaluate_selected(0)
~ compute_thematic_coherence()
~ compute_prestige_score()
~ classify_visitor_response()

== function evaluate_selected(index: int) => void ==
{ if index >= LEN(selected_ids):
    Raw metrics captured.
- else:
    ~ temp exhibit: Exhibit = exhibit_by_id(selected_ids[index])
    Selected exhibit {exhibit.id}: {exhibit.name} ({exhibit.theme}), score {exhibit.quality} / {exhibit.appeal}
    ~ total_quality = total_quality + exhibit.quality
    ~ total_appeal = total_appeal + exhibit.appeal
    { if exhibit.theme == "Renaissance":
        ~ renaissance_count = renaissance_count + 1
    - else:
        { if exhibit.theme == "Modern":
            ~ modern_count = modern_count + 1
        - else:
            { if exhibit.theme == "Scientific":
                ~ scientific_count = scientific_count + 1
            - else:
                ~ other_count = other_count + 1
            }
        }
    }
    ~ evaluate_selected(index + 1)
}

== function compute_thematic_coherence() => void ==
~ temp max_theme: int = renaissance_count
{ if modern_count > max_theme:
    ~ max_theme = modern_count
}
{ if scientific_count > max_theme:
    ~ max_theme = scientific_count
}
{ if other_count > max_theme:
    ~ max_theme = other_count
}
~ thematic_coherence = (total_quality / 2) + (total_appeal / 4) + (max_theme * 8) - (selected_count - max_theme) * 9
{ if thematic_coherence < 0:
    ~ thematic_coherence = 0
}
{ if thematic_coherence > 100:
    ~ thematic_coherence = 100
}

== function compute_prestige_score() => void ==
~ temp average_appeal: int = 0
~ temp average_quality: int = 0
{ if selected_count > 0:
    ~ average_appeal = total_appeal / selected_count
    ~ average_quality = total_quality / selected_count
}
~ prestige_score = average_quality + average_appeal + thematic_coherence
{ if prestige_score < 0:
    ~ prestige_score = 0
}

== function classify_visitor_response() => void ==
{ if thematic_coherence >= 85:
    { if prestige_score >= 180:
        ~ visitor_response = "High praise and long-form press coverage."
    - else:
        ~ visitor_response = "Respectful admiration and increased bookings."
    }
- else:
    { if thematic_coherence >= 65:
        { if prestige_score >= 160:
            ~ visitor_response = "Balanced approval and mild social buzz."
        - else:
            ~ visitor_response = "Visitors accept the flow, but few linger."
        }
    - else:
        { if prestige_score >= 140:
            ~ visitor_response = "Polite comments with a note to rebalance."
        - else:
            ~ visitor_response = "Guests leave confused and underwhelmed."
        }
    }
}

== function print_curation_report() => void ==
Curation report:
Total selected exhibits: {selected_count}
Theme counts:
Renaissance: {renaissance_count}
Modern: {modern_count}
Scientific: {scientific_count}
Other: {other_count}
Thematic coherence: {thematic_coherence}
Prestige score: {prestige_score}
Visitor response: {visitor_response}

== function exhibit_by_id(id: int) => Exhibit ==
~ temp index: int = exhibit_index[id]
~ return exhibit_catalog[index]

== function is_selected_id(id: int, scan: int) => bool ==
{ if scan >= LEN(selected_ids):
    ~ return false
- else:
    { if selected_ids[scan] == id:
        ~ return true
    - else:
        ~ return is_selected_id(id, scan + 1)
    }
}
