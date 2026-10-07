local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local ks = {}
for _, l in ipairs(lines(given.keywords:render())) do
  local k = trim(l):lower()
  if k:match("^[a-z]+$") then ks[#ks+1] = '"' .. k .. '"' end
end
if #ks == 0 then ks[1] = '"nothing"' end
return host.text(host.output, "cd repo && ubc query cypher --project docs --strict 'MATCH (n:dec) WITH n, size([k IN [" .. table.concat(ks, ", ") .. "] WHERE toLower(n.title) CONTAINS k OR toLower(n.statement) CONTAINS k OR toLower(n.content) CONTAINS k]) AS hits WHERE hits > 0 RETURN n.id, n.title, n.statement, hits ORDER BY hits DESC, n.id LIMIT 20'")
