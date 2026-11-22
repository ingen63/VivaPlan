# Withdrawal strategy

## @s-withdrawal Withdrawal strategy: overview
Keywords: withdrawal, strategy, retirement, guardrails, living expenses

The strategy determines **how money is withdrawn from wealth and reallocated after retirement**. There are six strategies, divided into two groups. The 3-bucket strategy can also be combined with guardrails:

**Spending is fixed, wealth fluctuates** (decision: how is the wealth invested?)

- [Standard](#f-wd-standard): cash reserve plus a fixed stock allocation
- [3 buckets](#f-wd-three_buckets): cash, bonds and stocks in three buckets
- [3 buckets with guardrails](#f-wd-three_buckets_guardrails): the buckets determine the investment, the guardrails the spending

**Spending follows wealth** (decision: how much do I spend?)

- [Fixed withdrawal rate](#f-wd-fixed_rate): a fixed share of wealth every year
- [Guardrails](#f-wd-guardrails): initial rate with guardrails (Guyton-Klinger)
- [VPW](#f-wd-vpw): variable percentage withdrawal, so that wealth lasts until a given age
- [Endowment](#f-wd-endowment): smoothed withdrawal (Yale/Tobin)

**Security versus comfort:** With the first two strategies, your spending stays as planned, but wealth can be depleted. With the four others, wealth is **never fully depleted**, but spending fluctuates with the stock market. Whether you want that is a personal question. With the [limits on living expenses](#f-wd-limits) you set a floor.

**When does the strategy apply?** Only **from retirement**, with two persons from the retirement of the **first** person. This **also applies to “Standard”**. Before that, no strategy applies: the app holds as much cash as entered under [Wealth & investment](#s-wealth) (adjusted for inflation) and invests the rest according to the stock allocation. If the second person is still working, their income counts toward “income and pensions”: with the strategies whose spending follows wealth, living expenses are then higher by this income. When it ends with the second retirement, living expenses drop and the withdrawal stays the same.

**Inflows take effect immediately:** With the four strategies whose spending follows wealth (with 3 buckets, instead: inflows only go to bucket 3; with 3 buckets with guardrails, both apply: the guardrails recalculate the withdrawal, the money goes to bucket 3), the app applies the strategy not only in January but **also in the month of an inflow**: for a one-off amount (for example an inheritance), a pension fund payout and the proceeds of a property sale. It then behaves as in January (the annual withdrawal is recalculated), and this applies from that month for the rest of the year, not retroactively. Several inflows in the same month result in **one** application after all inflows and purchases. “Standard” invests every month anyway.

**Sale with a subsequent purchase:** If the purchase of another property is planned for the month after the sale (fixed purchase month or “after sale”), the proceeds are **reserved** and the strategy is **not** applied in the month of the sale. In the month of the purchase, the purchase comes first, then the strategy is applied to the surplus. For a sale in December and a purchase in January, the order is purchase, then the usual January application (once). Reserved proceeds do **not** count toward the wealth from which the strategies calculate their withdrawal. The list of events and the section “Real estate” state what happened (“Sale proceeds reserved for a purchase in the following month”, “Withdrawal strategy applied after …”).

**Which one suits you?** See [Which strategy suits you?](#wd-compare). The [optimization](#s-optimize) provides a calculated answer.

## @wd-compare Which strategy suits you?
Keywords: comparison, recommendation, choice, pros and cons

| Strategy | Spending | Wealth | Good for | Drawback |
|---|---|---|---|---|
| Standard | fixed, as planned | can run out | simple planning, a strong need for security in spending | running out of money is possible, no mechanism against crises |
| 3 buckets | fixed, as planned | can run out | those who want to bridge crisis years from reserves | the rules are more complicated, the parameters must fit |
| 3 buckets with guardrails | follows wealth within guardrails | almost never runs out | predictable spending and crisis years bridged from reserves | many parameters, lower stock allocation |
| Fixed rate | follows wealth | never runs out | the simplest rule that never depletes wealth | large fluctuation of spending |
| Guardrails | follows wealth within guardrails | almost never runs out | predictable spending with correction in crises | cuts of 10 % possible |
| VPW | follows wealth, rises with age | depleted at the planning age | those who want to use up their wealth | spending fluctuates, the planning age is an assumption |
| Endowment | follows wealth with a delay | almost never runs out | steady spending | reacts slowly, can withdraw too much in a crash |

**A practical approach:** Start with “Standard” as the baseline, then run the [optimization](#s-optimize) over all strategies and look at how much consumption and how much security the alternatives bring.

## @f-wd-choice Choosing a strategy
Keywords: choice, card

Choose the strategy with the cards at the top of the section. Every card has an ⓘ with the detailed description. The inputs of the chosen strategy appear below.

The strategies whose spending follows wealth, and 3 buckets with guardrails, determine **living expenses from retirement**. The planned spending (section “Income & spending”) then only applies until retirement.

See [Overview](#s-withdrawal) and [Which strategy suits you?](#wd-compare).

## @f-wd-standard Standard
Keywords: reserve, stock allocation, rebalancing

**Spending is fixed and follows your plan.** Wealth is split as follows:

- The **cash** is kept at the [cash reserve](#f-reserve) (adjustable here; all other strategies use a fixed 3 months), expressed in months of uncovered spending (spending minus income and pensions).
- The **rest** is split into stocks and bonds according to the [stock allocation](#f-stockshare).
- The ratio of stocks to bonds is restored monthly (rebalancing).

Like all strategies, it only applies from retirement; before that, the cash rule from [Wealth & investment](#s-wealth) applies. Simple and transparent, but without protection against selling too much in a crisis.

## @f-wd-three_buckets 3-bucket strategy
Keywords: bucket, cash, bonds, stocks, refill, January, threshold

From **retirement**, wealth is divided into three buckets. Spending stays **as planned**:

| Bucket | Content | Target |
|---|---|---|
| Bucket 1 | Cash | years × net need, covers the current spending |
| Bucket 2 | Bonds | years × net need, buffer |
| Bucket 3 | Stocks | the rest |

**Inputs:**

- **Bucket 1 (years):** How many years of the net need are held as cash. Usual: 1 to 3.
- **Bucket 2 (years):** How many years are held as bonds. Usual: 3 to 8.
- **Sell from bucket 3 only if the stock return (12 months) ≥:** return criterion. What is measured is the **stock return of the last 12 months**, including the current month. In the first 12 months of the simulation, the months before the start also count: every return path brings them along (historical and bootstrap: the real months before; parametric: randomly drawn ones; fixed returns: the fixed rate). If they are missing because the time series starts later, the return since the start applies. If the criterion is not met, only **sales from bucket 3** in the January refill are blocked. At 0 %, the app only sells after a year of gains; negative values allow refilling even after slight losses.
- **Reserve in bucket 1 during the year (months):** If bucket 1 runs low during the year (case C), min(reserve, months until January) − 1 months of need remain in it after the month's spending. Default: 3.

**Rules:**

- All spending is paid from **bucket 1**. **No bucket falls below 0**; for the only exception, see ‘All buckets empty’.
- **Targets** (always from the net need, never from the bucket balance): target of bucket 1 = years × net need, target of bucket 2 = years × net need.
- **January refill (every January):** If the return criterion is met, bucket 1 is refilled from bucket 2, and the rest comes from bucket 3; then bucket 2 is refilled from bucket 3. If it is not met, nothing is sold from bucket 3, bucket 1 is only refilled from bucket 2 (if bucket 2 is not enough, bucket 1 stays below its target), and bucket 2 is not refilled. If bucket 1 or bucket 2 is above its target, the surplus goes to bucket 3.
- **A · Start of the withdrawal phase, pension fund lump sum, restart:** At the start (as soon as the **first** person is retired), all liquid wealth first goes to bucket 3, and so does a later pension fund lump sum. Then the refill runs as in January, **without** the return criterion and (during the year) without the surplus transfer to bucket 3. The targets are reduced by the months already passed this year ((years × 12 − months) / 12 × net need; start in March: 2 months). A runs at every trigger; several in the same month result in one run.
- **B · External inflows:** One-off inflows, 3a payouts, a property sale without a subsequent purchase and the proceeds of emergency sales go **only to bucket 3**. Buckets 1 and 2 are not refilled.
- **C · Look-ahead (every month):** The app checks whether bucket 1 covers the month's spending (recurring and one-off, taxes included). If not, the procedure depends on the return criterion:
  - **C1 (criterion met):** Same procedure as the January refill: bucket 1 from bucket 2, the rest from bucket 3, then bucket 2 from bucket 3. The targets are reduced by the months passed since January ((years × 12 − months) / 12 × net need; April = 3 months, December = 11), and the target of bucket 1 is additionally increased by the **one-off spending of this month**. There is **no surplus transfer** to bucket 3. Example April, bucket 1 = 2 years, bucket 2 = 1 year: target of bucket 1 = 1.75 × net need + one-off spending, target of bucket 2 = 0.75 × net need.
  - **C2 (criterion not met):** Bucket 1 is refilled from **bucket 2, then bucket 3** so that it covers the spending and afterwards still holds (min(reserve, months until January) − 1) months of need (“months until January” counts the current month: December = 1). Bucket 2 is not refilled.
  - If bucket 1 is still not enough for the month's payments after C1, C2 follows. If buckets 2 and 3 are not enough, the emergency sale follows (see ‘All buckets empty’).
- **One-off spending and targets:** In the January refill and in A, the target of bucket 1 is also increased by the month's one-off spending, so that a large one-off expense in January does not additionally trigger C. **Surplus transfers** to bucket 3 only happen in the January refill.
- **Real estate:** Sale proceeds for which a purchase is planned in the following month are **reserved** (they belong to no bucket). At the purchase, the reservation pays the equity first, and any rest comes from bucket 1 (via C); a surplus of the reservation goes to bucket 3. A purchase in January after a sale in December is settled before the January refill.
- **Order within the month:** regular income into bucket 1, inflows directly to their target (bucket 3 or the reservation, never first into bucket 1), property purchases (equity from the reservation; any rest counts as one-off spending of the month and does not burden bucket 1 beforehand), January refill or A, look-ahead (C) with **all** of the month's spending (recurring, one-off, rest of a purchase, amortizations and one-off payments of the mortgage, taxes), and then the spending is paid from bucket 1. Technically, the transfers are booked after the return step of the month; the table shows the balance before the month's spending.
- **All buckets empty:** If buckets 1, 2 and 3 are not enough, a property is sold in an emergency sale (proceeds to bucket 3, then C again). The emergency sale criterion of the mortgage logic applies in parallel and can trigger earlier. If there is no property left, bucket 1 may become negative, buckets 2 and 3 are frozen, and all income and inflows flow into bucket 1. As soon as wealth is positive again, A restarts.

**Net need (per year, planned values of the current year):** living expenses + housing + pillar 3a contributions + income tax − regular income (AHV pension, from 2026 with the 13th pension: 13 monthly pensions per year; PK pension; earned income as entered). Not counted are one-off spending, acquisitions, travel, amortizations, property purchases, special taxes (property gains tax, lump-sum tax and wealth tax) and one-off inflows (lump sum, inheritance, sale proceeds). The income tax is the tax on the annual income (pensions including the 13th AHV pension, and earned income). For married couples the already capped household AHV counts; the 13th pension is not capped again. If the net need is zero or negative, all targets are zero. At a trigger A, the months from the trigger on are extrapolated to 12 months, **income tax included**. This net need applies to all refills until the next January or the next trigger A, including the look-ahead C. If spending rises (inflation), the buckets grow with it. In the result view, the table “3-bucket strategy” shows the derivation (spending, deducted income, net need, targets), the return criterion, the case triggered (January, A, B, C) and all transfers.

**Idea:** After a stock market crash, no stocks have to be sold at low prices. Buckets 1 and 2 bridge several years until the stocks have recovered. The cost is the lower stock allocation (more bonds and cash) and therefore a tendency toward lower returns.

## @f-wd-three_buckets_guardrails 3 buckets with guardrails
Keywords: combination, buckets, guardrails, spending, investment, net need

On the card “3 buckets”, under **Spending from retirement**, you choose between “as planned” and “control with guardrails”. With guardrails:

- **The guardrails control spending.** Initial rate, inflation, guardrails and [limits](#f-wd-limits) work exactly as with the strategy [Guardrails](#f-wd-guardrails). With the same course of wealth, the withdrawal is the same in both.
- **The buckets control the investment.** All rules of the [3-bucket strategy](#f-wd-three_buckets) apply unchanged (January refill, A, B, C, return criterion, emergency sale, “all buckets empty”).
- **Net need and bucket targets** are calculated with the living expenses of the guardrails instead of the planned spending, after floor and ceiling. Without limits and without housing costs, the net need is the guardrails withdrawal plus income tax. The table “3-bucket strategy” marks the living expenses with “Guardrails”.
- **Order in January:** first the guardrails (inflation, guardrails), then net need and targets, then the refill. If the guardrail cuts the withdrawal by 10 %, the targets of buckets 1 and 2 also become 10 % smaller.
- **Start:** Buckets and guardrails start together with the retirement of the first person. First the initial withdrawal is set, then A divides the wealth.
- **Inflows:** The guardrails recalculate the withdrawal, the money goes to bucket 3. The bucket targets stay until the next January; the look-ahead (C), however, immediately uses the new monthly living expenses.
- **Two return measures:** The guardrails suspend the inflation adjustment after a loss year in the **calendar year**; the return criterion of the buckets measures the **last 12 months**. In January, these are the same.

**Example:** Initial rate 4 %, wealth 1 million, bucket 1 = 1 year, bucket 2 = 2 years, no pensions. The withdrawal is 40'000 per year, bucket 1 holds 40'000, bucket 2 80'000, the rest is in stocks. After a crash, the guardrail cuts to 36'000: in the next January, the targets are 36'000 and 72'000.

**Idea:** The buckets avoid selling stocks at low prices, the guardrails prevent withdrawing too much. In the [optimization](#s-optimize), the combination is called “3 buckets + guardrails”.

## @f-wd-fixed_rate Fixed withdrawal rate
Keywords: percent, rate, 4 percent, Trinity, constant rate

Every year, **the same rate** of liquid wealth is withdrawn. Spending thus **follows** wealth.

- Every **January** (and at retirement), the annual withdrawal is reset as **rate × liquid wealth**.
- The withdrawal is **living expenses + housing costs + pillar 3a contributions minus income and pensions** (before taxes). Living expenses follow from it. The planned spending from the section “Income & spending” only applies until retirement.
- **Wealth is never fully depleted**, because only a share is ever withdrawn. In return, spending fluctuates strongly: if wealth falls by 30 %, the withdrawal falls by 30 %.

**Which rate?** The often-cited ‘4 percent rule’ comes from a US study over 30 years with a fixed, inflation-adjusted withdrawal (**not** with a rate on the current wealth). For a rate on the current wealth, 3 % to 5 % is usual. Note: the rate applies to the **withdrawal**, i.e. to your spending **minus** AHV, pension fund pension and income. Living expenses are therefore rate × wealth **plus** pensions and income. With high pensions, spending thus depends much less on wealth than the rate suggests.

The **floor** of the [limits on living expenses](#f-wd-limits) prevents spending that is too low.

## @f-wd-guardrails Guardrails (Guyton-Klinger)
Keywords: guardrails, initial rate, upper, lower, adjustment, Guyton, Klinger

The withdrawal starts with an **initial rate** and is adjusted for inflation every year. **Guardrails** intervene when the withdrawal becomes too high or too low relative to wealth.

**Inputs** (default in parentheses):

- **Initial rate (5 %):** Share of liquid wealth that is withdrawn at retirement (of the first person). Sale proceeds reserved for a purchase do not count toward it, not even for the guardrails.
- **Upper guardrail (20 %):** If the current rate exceeds the initial rate by this share, the withdrawal is cut.
- **Lower guardrail (20 %):** If the current rate is below the initial rate by this share, the withdrawal is raised.
- **Adjustment (10 %):** By how much the withdrawal is cut or raised when a guardrail is breached.

**Procedure every January:**

1. The previous year's withdrawal is adjusted for **inflation**, **except** after a year of stock losses as long as the current rate is above the initial rate (no inflation adjustment in a crisis).
2. Then the current rate (withdrawal / wealth) is compared with the guardrails:
   - **Above initial rate × (1 + upper guardrail):** the withdrawal is cut by the adjustment.
   - **Below initial rate × (1 − lower guardrail):** the withdrawal is raised by the adjustment.

**Example:** Initial rate 5 %, upper guardrail 20 %, adjustment 10 %. If wealth falls so sharply that the current rate rises above 6 %, the app cuts the withdrawal by 10 %.

**Idea:** You spend a lot at the beginning (the initial rate is often higher than with a fixed rate), but accept saving in crises. The cuts are rare and small; spending stays **more predictable** than with a pure fixed rate.

As with the fixed rate, the withdrawal is **living expenses + housing costs + pillar 3a contributions minus income and pensions** (before taxes). [Limits on living expenses](#f-wd-limits) are possible.

## @f-wd-vpw Variable percentage withdrawal (VPW)
Keywords: VPW, planning age, real return, annuity, Bogleheads

Every January, the share of liquid wealth is withdrawn that **uses it up evenly until the planning age** at an assumed **real return**. This is an **annuity**.

**Inputs:**

- **Planning age** (age of the main person, default 100): Wealth should last until then. Choose it high enough to keep a safety margin.
- **Real return** (default 2 %): The real annual return on wealth that the strategy assumes. An assumption that is too high leads to withdrawals that use up wealth faster.

**Properties:**

- The withdrawn **rate rises with age** because the remaining time gets shorter (for example at a real return of 0 % with 25 years left: 4 %; with 10 years: 10 %).
- At the planning age, wealth would be used up. The strategy is therefore built for **‘the money should be gone at the end’**, not for leaving an inheritance.
- Spending follows wealth and fluctuates accordingly. A floor limits this.

## @f-wd-endowment Endowment rule (Yale/Tobin)
Keywords: smoothing, target rate, Yale, Tobin, smoothed

The withdrawal is a **weighted average** of the previous year (with inflation) and a rate on the current wealth:

> Withdrawal in January = (1 − smoothing) × previous year's withdrawal with inflation + smoothing × target rate × current wealth

**Inputs:**

- **Target rate** (default 4 %): The rate the withdrawal approaches in the long run.
- **Smoothing** (default 30 %): Weight of the current wealth. At 100 %, it is the [fixed rate](#f-wd-fixed_rate); at 0 %, only the previous year is carried forward with inflation (like a fixed withdrawal).

**Effect:** With little smoothing, spending changes little from year to year. But it follows wealth **slowly**. If wealth falls sharply, too much is withdrawn in the short term, and this only evens out over years. Foundations and universities use this rule to achieve stable distributions.

## @f-wd-limits Limits on living expenses
Keywords: floor, ceiling, lower limit, upper limit, minimum spending

With the strategies that set living expenses themselves (fixed rate, guardrails, VPW, endowment, 3 buckets with guardrails), you can set a **floor** and a **ceiling**. Both are **shares of the planned spending**. **0 means no limit.**

- **Floor:** Living expenses never fall below this share of the planned spending (in today's prices, adjusted for inflation). Example: 80 % means you spend at least 80 % of what you planned, even if the strategy would allow less. **Then wealth can be depleted.** The floor makes the price of security in spending visible.
- **Ceiling:** Living expenses never rise above this share. Example: 130 % prevents you from spending considerably more than planned in good years.

The limits are one way of protecting **fixed commitments** (housing, health) against the flexible withdrawal. In the [optimization](#s-optimize), the limits stay as in the plan. In the result of the optimization and of the strategy comparison, the column **“At floor”** shows in how many months from retirement the floor sets the living expenses (see [Reading the result](#o-read)).

The limits do not apply to “Standard” and to “3 buckets” with spending as planned, because spending is fixed there anyway.
