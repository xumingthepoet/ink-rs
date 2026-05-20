=== module game ===

STRUCT Bidder {
    id: string
    budget: int
    priority: int
}

STRUCT AuctionItem {
    id: string
    name: string
    reserve: int
}

STRUCT BidProfile {
    bidder: string
    item: string
    baseline: int
}

VAR bidders: Bidder[] = [
    %Bidder{id: "Lena", budget: 30, priority: 2},
    %Bidder{id: "Quinn", budget: 35, priority: 6},
    %Bidder{id: "Rook", budget: 28, priority: 3},
    %Bidder{id: "Vale", budget: 40, priority: 1}
]

VAR items: AuctionItem[] = [
    %AuctionItem{id: "helm", name: "Aegis Helm", reserve: 20},
    %AuctionItem{id: "arrow", name: "Signal Arrows", reserve: 12},
    %AuctionItem{id: "potion", name: "Fortune Draught", reserve: 16}
]

VAR bid_profiles: BidProfile[] = [
    %BidProfile{ bidder: "Lena", item: "helm", baseline: 20 },
    %BidProfile{ bidder: "Quinn", item: "helm", baseline: 20 },
    %BidProfile{ bidder: "Rook", item: "helm", baseline: 11 },
    %BidProfile{ bidder: "Vale", item: "helm", baseline: 13 },

    %BidProfile{ bidder: "Lena", item: "arrow", baseline: 8 },
    %BidProfile{ bidder: "Quinn", item: "arrow", baseline: 9 },
    %BidProfile{ bidder: "Rook", item: "arrow", baseline: 9 },
    %BidProfile{ bidder: "Vale", item: "arrow", baseline: 8 },

    %BidProfile{ bidder: "Lena", item: "potion", baseline: 9 },
    %BidProfile{ bidder: "Quinn", item: "potion", baseline: 9 },
    %BidProfile{ bidder: "Rook", item: "potion", baseline: 12 },
    %BidProfile{ bidder: "Vale", item: "potion", baseline: 11 }
]

VAR winning_bid: Dict<string, int> = %{
    "helm": 0,
    "arrow": 0,
    "potion": 0
}

VAR winning_bidder: Dict<string, string> = %{
    "helm": "unawarded",
    "arrow": "unawarded",
    "potion": "unawarded"
}

VAR bid_met_reserve: Dict<string, bool> = %{
    "helm": false,
    "arrow": false,
    "potion": false
}

VAR total_spend: int = 0
VAR total_bids: int = 0
VAR items_sold: int = 0

== main ==
Auction with budgets and priorities.
~ print_bidders("Opening budgets")
~ run_auction(0)
~ print_bidders("Closing budgets")
~ print_ledger()
Auction result:
Total bids: {total_bids}
Items sold: {items_sold}
Total spend: {total_spend}
-> DONE

== function run_auction(item_index: int) => void ==
{ if item_index < LEN(items):
    ~ temp item: AuctionItem = items[item_index]
    Auction lot: {item.name}
    ~ reset_item(item.id)
    ~ evaluate_bidders_for_item(item_index, 0)
    ~ finalize_item(item.id)
    ~ run_auction(item_index + 1)
}

== function evaluate_bidders_for_item(item_index: int, bidder_index: int) => void ==
{ if bidder_index < LEN(bidders):
    ~ temp bidder: Bidder = bidders[bidder_index]
    ~ temp item: AuctionItem = items[item_index]
    ~ temp profile_index: int = find_bid_profile(item.id, bidder.id, 0)
    { if profile_index == -1:
        {bidder.id} has no profile for {item.id}.
    - else:
        ~ temp profile: BidProfile = bid_profiles[profile_index]
        ~ temp computed_bid: int = profile.baseline + bidder.priority
        ~ temp prior_leader: string = winning_bidder[item.id]
        ~ total_bids = total_bids + 1
        { if bidder.budget >= computed_bid:
            { if prior_leader == "unawarded":
                ~ winning_bidder[item.id] = bidder.id
                ~ winning_bid[item.id] = computed_bid
                ~ bid_met_reserve[item.id] = computed_bid >= item.reserve
                {bidder.id} opens the book on {item.name} at {computed_bid}.
            - else:
                { if computed_bid > winning_bid[item.id]:
                    ~ winning_bidder[item.id] = bidder.id
                    ~ winning_bid[item.id] = computed_bid
                    ~ bid_met_reserve[item.id] = computed_bid >= item.reserve
                    {bidder.id} outbids {prior_leader} at {computed_bid}.
                - else:
                    { if computed_bid == winning_bid[item.id]:
                        {bidder.id} matches current top with {computed_bid}.
                    - else:
                        {bidder.id} bids {computed_bid} but does not lead.
                    }
                }
            }
        - else:
            {bidder.id} cannot afford {computed_bid} on {item.name} (budget {bidder.budget}).
        }
    }
    ~ evaluate_bidders_for_item(item_index, bidder_index + 1)
}

== function finalize_item(item_id: string) => void ==
~ temp winner: string = winning_bidder[item_id]
~ temp price: int = winning_bid[item_id]
{ if winner == "unawarded":
    {item_id} closes with no bids.
 - else:
    { if bid_met_reserve[item_id]:
        ~ temp winner_index: int = find_bidder_index(winner, 0)
        ~ bidders[winner_index].budget = bidders[winner_index].budget - price
        ~ total_spend = total_spend + price
        ~ items_sold = items_sold + 1
        {winner} wins {item_id} for {price}. Budget {bidders[winner_index].budget} remains.
    - else:
        {item_id} reached top bid {price} from {winner}, but reserve not met.
        ~ winning_bidder[item_id] = "unawarded"
    }
}

== function find_bid_profile(item_id: string, bidder_id: string, index: int) => int ==
{ if index >= LEN(bid_profiles):
    ~ return -1
- else:
    { if bid_profiles[index].item == item_id && bid_profiles[index].bidder == bidder_id:
        ~ return index
    - else:
        ~ return find_bid_profile(item_id, bidder_id, index + 1)
    }
}

== function find_bidder_index(id: string, index: int) => int ==
{ if index >= LEN(bidders):
    ~ return -1
- else:
    { if bidders[index].id == id:
        ~ return index
    - else:
        ~ return find_bidder_index(id, index + 1)
    }
}

== function reset_item(item_id: string) => void ==
~ winning_bid[item_id] = 0
~ winning_bidder[item_id] = "unawarded"
~ bid_met_reserve[item_id] = false

== function print_bidders(label: string) => void ==
-- {label} --
~ print_bidder_entry(0)

== function print_bidder_entry(index: int) => void ==
    { if index < LEN(bidders):
    ~ temp bidder: Bidder = bidders[index]
    {bidder.id} budget {bidder.budget} priority {bidder.priority}
    ~ print_bidder_entry(index + 1)
}

== function print_ledger() => void ==
Winning ledger:
~ print_ledger_entry(0)

== function print_ledger_entry(index: int) => void ==
{ if index < LEN(items):
    ~ temp item: AuctionItem = items[index]
    ~ temp winner: string = winning_bidder[item.id]
    ~ temp price: int = winning_bid[item.id]
    { if winner == "unawarded":
        {item.name} unsold.
    - else:
        {item.name} sold to {winner} for {price}.
        { if bid_met_reserve[item.id]:
            reserve met
        - else:
            reserve missed
        }
    }
    ~ print_ledger_entry(index + 1)
}
