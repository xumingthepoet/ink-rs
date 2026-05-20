=== module game ===

STRUCT Supplier {
    name: string
    base_cost: int
    sell_price: int
    freight_rate: int
    replenish: int
    reliability: int
}

VAR suppliers: Supplier[] = [
    %Supplier{ name: "North Caravans", base_cost: 7, sell_price: 13, freight_rate: 2, replenish: 6, reliability: 87 },
    %Supplier{ name: "Tidebound Docks", base_cost: 9, sell_price: 16, freight_rate: 3, replenish: 8, reliability: 76 },
    %Supplier{ name: "Glassroad Trust", base_cost: 6, sell_price: 12, freight_rate: 3, replenish: 5, reliability: 92 },
    %Supplier{ name: "Ashfall Bins", base_cost: 8, sell_price: 15, freight_rate: 4, replenish: 7, reliability: 81 }
]

VAR supplier_stock: Dict<string, int> = %{
    "North Caravans": 64,
    "Tidebound Docks": 72,
    "Glassroad Trust": 59,
    "Ashfall Bins": 53
}

VAR demand_plan: int[] = [31, 42, 27, 53, 29, 46, 34, 39, 41, 22]
VAR route_delay: int[] = [1, 3, 2, 4, 1, 5, 2, 3, 1, 4]

VAR opening_cash: int = 4200
VAR cash: int = 4200
VAR warehouse_stock: int = 22
VAR backlog: int = 0
VAR fulfilled_total: int = 0
VAR shortfall_total: int = 0
VAR stale_loss_units: int = 0
VAR gross_revenue: int = 0
VAR material_spent: int = 0
VAR freight_spent: int = 0
VAR late_shipments: int = 0
VAR delayed_units: int = 0
VAR stock_waste: int = 0
VAR contracts_completed: int = 0

== main ==
Mercantile supply chain report.

Opening cash: {opening_cash}
Opening warehouse: {warehouse_stock} crates
~ run_supply_chain(0)
~ print_financial_summary()
-> DONE

== function run_supply_chain(week: int) => void ==
{ if week >= LEN(demand_plan):
    Supply cycle closed.
- else:
    ~ temp supplier_index: int = week % LEN(suppliers)
    ~ temp supplier: Supplier = suppliers[supplier_index]
    ~ temp demand: int = demand_plan[week]
    ~ temp transit_days: int = route_delay[week]
    ~ process_contract(week + 1, supplier, demand, transit_days)
    ~ run_supply_chain(week + 1)
}

== function process_contract(week: int, supplier: Supplier, demand: int, transit_days: int) => void ==
~ contracts_completed = contracts_completed + 1
~ temp prior_backlog: int = backlog
~ temp scheduled_demand: int = demand + prior_backlog
~ temp current_stock: int = supplier_stock[supplier.name]
~ temp replenished_stock: int = current_stock + supplier.replenish
~ supplier_stock[supplier.name] = replenished_stock

Contract week {week}
Supplier: {supplier.name}
Requested demand (including backlog): {scheduled_demand}
Stock after replenishment: {replenished_stock}
Reliability index: {supplier.reliability}%, route delay {transit_days}

~ temp delay_loss: int = 0
~ temp delay_penalty: int = 0
{ if transit_days > 2:
    ~ late_shipments = late_shipments + 1
    ~ delay_loss = transit_days - 2
    ~ delay_penalty = delay_loss * (3 - (supplier.reliability / 50))
    { if delay_penalty < 0:
        ~ delay_penalty = 0
    }
    ~ scheduled_demand = scheduled_demand - delay_penalty
    { if scheduled_demand < 0:
        ~ scheduled_demand = 0
    }
    ~ stock_waste = stock_waste + delay_penalty
    Shipment delay causes {delay_penalty} demand units to decay before arrival.
- else:
    Shipment holds by schedule.
}

~ temp dispatch_capacity: int = min_two(replenished_stock, supplier.reliability + 10)
~ temp fulfilled_this_week: int = min_two(dispatch_capacity, scheduled_demand)
~ temp remaining_stock: int = replenished_stock - fulfilled_this_week
~ supplier_stock[supplier.name] = remaining_stock

~ backlog = scheduled_demand - fulfilled_this_week
{ if backlog < 0:
    ~ backlog = 0
}

~ temp stale_goods: int = 0
{ if transit_days >= 3:
    ~ stale_goods = fulfilled_this_week / (transit_days - 1)
    { if stale_goods > fulfilled_this_week:
        ~ stale_goods = fulfilled_this_week
    }
    ~ stale_loss_units = stale_loss_units + stale_goods
}

~ temp saleable_units: int = fulfilled_this_week - stale_goods

~ temp revenue: int = saleable_units * supplier.sell_price
~ temp material_cost: int = fulfilled_this_week * supplier.base_cost
~ temp freight: int = supplier.freight_rate * transit_days
~ temp net_week: int = revenue - (material_cost + freight)

~ gross_revenue = gross_revenue + revenue
~ material_spent = material_spent + material_cost
~ freight_spent = freight_spent + freight
~ fulfilled_total = fulfilled_total + fulfilled_this_week
~ shortfall_total = shortfall_total + backlog
~ delayed_units = delayed_units + stale_goods

~ temp before_cash: int = cash
~ cash = cash - material_cost - freight + revenue

~ temp warehouse_gain: int = saleable_units
~ warehouse_stock = warehouse_stock + warehouse_gain

Week shipment result:
Shipped units: {fulfilled_this_week}
Stale units: {stale_goods}
Units sold: {saleable_units}
Demand backlog carried: {backlog}
Net this contract: {net_week}
Cash was updated from {before_cash} to {cash}.
Warehouse stock now: {warehouse_stock}
Remaining supplier stock {supplier.name}: {remaining_stock}
{ if backlog > 0:
    Remaining demand missed this cycle: {backlog}
- else:
    Demand met for this contract.
}

== function print_financial_summary() => void ==
Profit and loss snapshot.
Opening cash {opening_cash}
Gross shipment revenue {gross_revenue}
Material spend {material_spent}
Freight spend {freight_spent}
Fulfillment volume {fulfilled_total}
Sale losses from staleness {stale_loss_units}
Demand shortfall {shortfall_total}
Stock decay losses from delay {stock_waste}
Late contracts {late_shipments}
Contracts completed {contracts_completed}
Final cash {cash}
Net performance {cash - opening_cash}
Restoration margin {gross_revenue - (material_spent + freight_spent)}
Warehouse final {warehouse_stock}

== function min_two(a: int, b: int) => int ==
{ if a < b:
    ~ return a
- else:
    ~ return b
}
