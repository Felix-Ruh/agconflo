local given, host = ...
local help = require('help')
local prompt = host.text(host.output, table.concat({
  help.NO_TOOLS .. "Write the pull request for this change: a title in the imperative, then what changed and why, what was checked (the final check and the loop's verdicts), and every open defect the notes list, said plainly. If the final check says FAIL or the notes list an open defect, say in the first sentence that the change is not ready.",
  "\n\n## brief\n\n" .. given.brief:render(),
  "\n\n## diff\n\n" .. given.diff:render(),
  "\n\n## final\n\n" .. given.final:render(),
  "\n\n## verdict\n\n" .. given.verdict:render()
}))
return host.text(host.output, host.complete('describing', prompt):render())
