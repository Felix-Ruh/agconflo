local given, host = ...
local ids, seen = {}, {}
for id in given.brief:render():gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
  if not seen[id] then seen[id] = true; ids[#ids+1] = '"' .. id .. '"' end
end
if #ids == 0 then ids[1] = '"NONE"' end
return host.text(host.output, "cd repo && ubc query cypher --project docs --strict 'MATCH (n) WHERE n.id IN [" .. table.concat(ids, ", ") .. "] RETURN n.id, n.title, coalesce(n.statement, n.observation) AS says ORDER BY n.id'")
