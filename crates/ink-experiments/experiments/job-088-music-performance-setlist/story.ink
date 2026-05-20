=== module game ===

STRUCT Song {
    title: string
    energy: int
    stamina_cost: int
    crowd_pull: int
    allows_encore: bool
}

VAR setlist_order: int[] = [0, 1, 2, 3, 4]

VAR songs: Song[] = [
    %Song{
        title: "Open Path Waltz",
        energy: 4,
        stamina_cost: 6,
        crowd_pull: 9,
        allows_encore: false
    },
    %Song{
        title: "Lantern March",
        energy: 6,
        stamina_cost: 8,
        crowd_pull: 11,
        allows_encore: false
    },
    %Song{
        title: "Rising Drumline",
        energy: 9,
        stamina_cost: 12,
        crowd_pull: 15,
        allows_encore: false
    },
    %Song{
        title: "Fading Ember Ballad",
        energy: 3,
        stamina_cost: 5,
        crowd_pull: 6,
        allows_encore: false
    },
    %Song{
        title: "Midnight Finale",
        energy: 8,
        stamina_cost: 11,
        crowd_pull: 17,
        allows_encore: true
    }
]

VAR encore_song: Song = %Song{
    title: "Encore: Bellfire Riff",
    energy: 7,
    stamina_cost: 9,
    crowd_pull: 14,
    allows_encore: false
}

VAR crowd_mood: int = 38
VAR performer_stamina: int = 72
VAR swaps: int = 0
VAR songs_performed: int = 0
VAR encore_unlocked: bool = false
VAR reception_points: int = 0

== main ==
The hall opens for a night run.
~ announce_order()
~ arrange_setlist(0)
~ perform_setlist(0)
~ maybe_encore()
~ final_reception()
-> DONE

== function announce_order() => void ==
Tonight's initial slots are set.

== function arrange_setlist(pass: int) => void ==
{ if pass >= 2:
    ~ return
- else:
    ~ improve_pairing(0)
    ~ arrange_setlist(pass + 1)
}

== function improve_pairing(index: int) => void ==
{ if index >= LEN(setlist_order) - 1:
    ~ return
- else:
    ~ temp left_id: int = setlist_order[index]
    ~ temp right_id: int = setlist_order[index + 1]
    ~ temp left_score: int = songs[left_id].energy + songs[left_id].crowd_pull
    ~ temp right_score: int = songs[right_id].energy + songs[right_id].crowd_pull
    { if right_score > left_score:
        ~ setlist_order[index] = right_id
        ~ setlist_order[index + 1] = left_id
        ~ swaps = swaps + 1
    }
    ~ improve_pairing(index + 1)
}

== function perform_setlist(index: int) => void ==
{ if index >= LEN(setlist_order):
    ~ return
- else:
    ~ temp song_id: int = setlist_order[index]
    ~ temp song: Song = songs[song_id]
    Song queued: {song.title}.
    { if performer_stamina < song.stamina_cost:
        Performance falters before {song.title}.
        ~ crowd_mood = crowd_mood - 4
    - else:
        ~ performer_stamina = performer_stamina - song.stamina_cost
        ~ crowd_mood = crowd_mood + song.crowd_pull - (song.energy / 2)
        ~ reception_points = reception_points + song.crowd_pull
        ~ songs_performed = songs_performed + 1
        ~ log_song_outcome(song.title, song.stamina_cost)
    }
    ~ normalize_mood()
    ~ perform_setlist(index + 1)
}

== function log_song_outcome(title: string, cost: int) => void ==
Energy cost {cost} drops stamina by that amount.

== function normalize_mood() => void ==
{ if crowd_mood > 100:
    ~ crowd_mood = 100
- else:
    { if crowd_mood < 0:
        ~ crowd_mood = 0
    }
}

== function maybe_encore() => void ==
{ if encore_unlocked == true:
    ~ return
- else:
    { if crowd_mood >= 78 and performer_stamina >= 20 and songs_performed >= 4:
        ~ encore_unlocked = true
        The crowd requests an encore.
        ~ perform_encore()
    - else:
        The crowd stays seated, no encore unlock.
    }
}

== function perform_encore() => void ==
~ performer_stamina = performer_stamina - encore_song.stamina_cost
{ if performer_stamina < 0:
    ~ performer_stamina = 0
    The vocalist cannot hold the encore fully.
- else:
    ~ crowd_mood = crowd_mood + encore_song.crowd_pull
    ~ songs_performed = songs_performed + 1
    ~ reception_points = reception_points + encore_song.crowd_pull
    Encore song lands with bright response.
}
~ normalize_mood()
}

== function final_reception() => void ==
Performance ends.
Setlist swaps: {swaps}
Songs fully performed: {songs_performed}
Audience mood: {crowd_mood}
Performer stamina left: {performer_stamina}
Crowd reception index: {reception_points}
{ if crowd_mood >= 86 and performer_stamina >= 16:
    Final reception: roaring approval.
- else:
    { if crowd_mood >= 72:
        Final reception: strong approval.
    - else:
        { if crowd_mood >= 58:
            Final reception: mixed but warm.
        - else:
            Final reception: tepid response.
        }
    }
}
{ if encore_unlocked:
    Encore was unlocked and delivered.
- else:
    Encore remained unavailable.
}
