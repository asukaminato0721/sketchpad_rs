// Finds direct references to the GSP4 file signature in the original executable.
//@category Sketchpad

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.mem.Memory;
import ghidra.program.model.mem.MemoryBlock;
import ghidra.program.model.symbol.Reference;

public class Gsp4Xrefs extends GhidraScript {
    @Override
    public void run() throws Exception {
        Memory memory = currentProgram.getMemory();
        byte[] signature = {'G', 'S', 'P', '4'};
        int foundCount = 0;
        for (MemoryBlock block : memory.getBlocks()) {
            Address address = block.getStart();
            while (address != null && address.compareTo(block.getEnd()) <= 0) {
                monitor.checkCancelled();
                Address match = memory.findBytes(address, signature, null, true, monitor);
                if (match == null || match.compareTo(block.getEnd()) > 0) {
                    break;
                }
                foundCount++;
                println("GSP4 signature at " + match + " in " + block.getName());
                Instruction containing = getInstructionContaining(match);
                Function owner = getFunctionContaining(match);
                println("  instruction: " + (containing == null ? "<none>" : containing));
                println("  function: " + (owner == null ? "<none>" : owner.getName() + " at " + owner.getEntryPoint()));
                if (owner != null) {
                    Reference[] callers = getReferencesTo(owner.getEntryPoint());
                    println("  function entry references: " + callers.length);
                    for (int i = 0; i < Math.min(callers.length, 12); i++) {
                        println("    " + callers[i].getFromAddress());
                    }
                }
                Reference[] references = getReferencesTo(match);
                println("  " + references.length + " direct reference(s)");
                for (Reference reference : references) {
                    Address source = reference.getFromAddress();
                    Function function = getFunctionContaining(source);
                    println("  " + source + ": " +
                        (function == null ? "<no function>" : function.getName()));
                    Instruction instruction = getInstructionAt(source);
                    if (instruction != null) {
                        println("    " + instruction);
                    }
                }
                try {
                    address = match.add(1);
                } catch (Exception exception) {
                    break;
                }
            }
        }
        println("Total GSP4 signatures: " + foundCount);
    }
}
