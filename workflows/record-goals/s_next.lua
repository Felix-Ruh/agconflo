local given, host = ...
-- Round again while the judge's verdict names the goal and pass to take,
-- which `head` reads from it; on to the final steps once it names none.
if ("\n" .. given.verdict:render()):match("\ngoal %d+\npass %d+\n") then
  host.route({ "head", "judge" })
else
  host.route({ "cmd_fin", "describe", "approval" })
end
