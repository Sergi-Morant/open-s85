// Decompile every function of the imported program into one text file, for fast grepping.
//
// analyzeHeadless <projectDir> <projectName> -import <program.exe> -overwrite \
//     -scriptPath <this dir> -postScript ExportDecompiled.java <out.c>
//
// The output is derived from someone else's software: keep it private, never commit it.
import java.io.PrintWriter;

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Function;

public class ExportDecompiled extends GhidraScript {
    @Override
    public void run() throws Exception {
        String out = getScriptArgs()[0];
        DecompInterface ifc = new DecompInterface();
        ifc.openProgram(currentProgram);
        int n = 0;
        try (PrintWriter w = new PrintWriter(out, "UTF-8")) {
            for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
                if (monitor.isCancelled()) {
                    break;
                }
                DecompileResults r = ifc.decompileFunction(f, 60, monitor);
                w.println("// ==== " + f.getEntryPoint() + " " + f.getName(true));
                if (r != null && r.decompileCompleted()) {
                    w.println(r.getDecompiledFunction().getC());
                } else {
                    w.println("// decompile failed: " + (r == null ? "null" : r.getErrorMessage()));
                }
                n++;
            }
        }
        println("exported " + n + " functions to " + out);
    }
}
