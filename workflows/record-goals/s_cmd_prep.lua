local given, host = ...
local g = given.goal:render()
local statement = g:match("STATEMENT: ([^\n]*)") or ""

local first = g:match("^GOAL 1 OF") and g:match("\nPASS 1\n")
local cmd
if first then cmd = "git -C repo reset -q --hard && git -C repo clean -fdq"
elseif g:match("\nSTAGE yes\n") then cmd = "git -C repo add -A && git -C repo checkout -q -- . && git -C repo clean -fdq"
else cmd = "git -C repo checkout -q -- . && git -C repo clean -fdq" end
return host.text(host.output, cmd .. " && git -C repo status --porcelain && echo ready")
