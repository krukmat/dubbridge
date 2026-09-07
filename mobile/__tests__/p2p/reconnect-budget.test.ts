import {
  createReconnectBudget,
  recordDisconnect,
} from "../../src/p2p/runtime/reconnect-budget";

describe("reconnect-budget", () => {
  it("HP-B2.b: the first disconnect within budget reports may-retry", () => {
    const budget = createReconnectBudget(1);
    const { decision } = recordDisconnect(budget);
    expect(decision).toBe("may-retry");
  });

  it("EC-B2.b-1: zero budget immediately returns exhausted", () => {
    const budget = createReconnectBudget(0);
    const { decision } = recordDisconnect(budget);
    expect(decision).toBe("exhausted");
  });

  it("EC-B2.b-2: exhaustion ordering and monotonicity", () => {
    const budget = createReconnectBudget(1);

    const first = recordDisconnect(budget);
    expect(first.decision).toBe("may-retry");
    expect(first.budget.usedRetries).toBe(1);

    const second = recordDisconnect(first.budget);
    expect(second.decision).toBe("exhausted");
    expect(second.budget.usedRetries).toBe(2);

    const third = recordDisconnect(second.budget);
    expect(third.decision).toBe("exhausted");
    expect(third.budget.usedRetries).toBe(3);
  });
});