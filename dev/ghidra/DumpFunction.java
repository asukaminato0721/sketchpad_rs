// Decompile selected function entry addresses in the original Sketchpad binary.
//@category Sketchpad

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Function;

public class DumpFunction extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] addresses = getScriptArgs();
        if (addresses.length == 0) {
            addresses = new String[] {"00416230", "00416610", "004f59d0"};
        }
        DecompInterface decompiler = new DecompInterface();
        decompiler.openProgram(currentProgram);
        try {
            for (String address : addresses) {
                Function function = getFunctionContaining(toAddr(address));
                if (function == null) {
                    println(address + ": no function entry");
                    continue;
                }
                DecompileResults result = decompiler.decompileFunction(function, 30, monitor);
                println("===== " + address + " " + function.getName() + " =====");
                if (!result.decompileCompleted()) {
                    println("Decompile failed: " + result.getErrorMessage());
                    continue;
                }
                println(result.getDecompiledFunction().getC());
            }
        } finally {
            decompiler.dispose();
        }
    }
}
