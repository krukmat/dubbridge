export interface ReconnectBudget {
  readonly maxRetries: number;
  readonly usedRetries: number;
}

export function createReconnectBudget(maxRetries: number): ReconnectBudget {
  return {
    maxRetries,
    usedRetries: 0,
  };
}

export type ReconnectDecision = "may-retry" | "exhausted";

export function recordDisconnect(budget: ReconnectBudget): {
  decision: ReconnectDecision;
  budget: ReconnectBudget;
} {
  const newUsedRetries = budget.usedRetries + 1;
  const decision: ReconnectDecision =
    newUsedRetries <= budget.maxRetries ? "may-retry" : "exhausted";

  return {
    decision,
    budget: {
      maxRetries: budget.maxRetries,
      usedRetries: newUsedRetries,
    },
  };
}