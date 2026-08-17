use agent_policy::merge::merge_policies;
use agent_policy::schema::{Decision, ToolClass};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_merge_never_widens(
        limit1 in 0..1000u32,
        limit2 in 0..1000u32,
        bool1 in any::<bool>(),
        bool2 in any::<bool>(),
    ) {
        let mut base = agent_policy::presets::standard_preset();
        let mut over = agent_policy::presets::standard_preset();

        base.commands.max_runtime_seconds = limit1;
        over.commands.max_runtime_seconds = limit2;

        base.filesystem.allow_outside_workspace = bool1;
        over.filesystem.allow_outside_workspace = bool2;

        let merged = merge_policies(&base, &over);

        // Assert limits are min()
        assert_eq!(merged.commands.max_runtime_seconds, std::cmp::min(limit1, limit2));

        // Assert boolean allow-flags use AND (restrictive)
        assert_eq!(merged.filesystem.allow_outside_workspace, bool1 && bool2);
    }
}

#[test]
fn test_presets() {
    let standard = agent_policy::presets::standard_preset();
    assert_eq!(
        standard.tools.default_decision.get(&ToolClass::Write),
        Some(&Decision::NeedsApproval)
    );

    let regulated = agent_policy::presets::regulated_preset();
    assert!(regulated.agent.require_plan_before_write);
    assert_eq!(regulated.models.require_managed_gateway, Some(true));
}
