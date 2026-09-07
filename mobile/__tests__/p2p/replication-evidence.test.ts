import { redactReplicationEvidence } from "../../src/p2p/proof/replication-evidence";

describe("redactReplicationEvidence", () => {
  it("HP-B2.d-i strips all four sensitive keys when present together", () => {
    const input = {
      topic: "some-topic",
      fixture: "some-fixture",
      fixtureContent: "some-content",
      discoveryKey: "some-key",
      role: "seed",
    };
    const output = redactReplicationEvidence(input);
    expect(!("topic" in output)).toBe(true);
    expect(!("fixture" in output)).toBe(true);
    expect(!("fixtureContent" in output)).toBe(true);
    expect(!("discoveryKey" in output)).toBe(true);
    expect("role" in output).toBe(true);
  });

  it("EC-B2.d-i preserves non-sensitive fields unchanged", () => {
    const input = {
      role: "seed",
      byte_count: 42,
      capability: "discover-and-replicate",
    };
    const output = redactReplicationEvidence(input);
    expect(output.role).toBe("seed");
    expect(output.byte_count).toBe(42);
    expect(output.capability).toBe("discover-and-replicate");
    expect(Object.keys(output)).toEqual(["role", "byte_count", "capability"]);
  });

  it("EC-B2.d-i returns an empty object for empty input", () => {
    const output = redactReplicationEvidence({});
    expect(Object.keys(output)).toHaveLength(0);
  });
});