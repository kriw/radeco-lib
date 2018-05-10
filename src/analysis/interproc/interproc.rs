//! Fills out the call summary information for `RFunction`

use std::collections::HashSet;
use analysis::interproc::transfer::InterProcAnalysis;
use frontend::radeco_containers::{RadecoFunction, RadecoModule};

#[derive(Debug)]
pub struct InterProcAnalyzer<'a, T>
    where T: InterProcAnalysis
{
    analyzed: HashSet<u64>,
    rmod: &'a mut RadecoModule,
    analyzer: T,
}

pub fn analyze_module<'a, A>(ssa: &'a mut RadecoModule)
    where A: InterProcAnalysis {
    let mut ipa = InterProcAnalyzer::<'a, A>::new(ssa);
    let fs = ipa.rmod.functions.clone();
    for (_, f) in fs {
        ipa.analyze_function(f.offset);
    }
}

impl<'a, T> InterProcAnalyzer<'a, T>
    where T: InterProcAnalysis
{
    pub fn new(rmod: &'a mut RadecoModule) -> InterProcAnalyzer<'a, T> {
        InterProcAnalyzer {
            analyzed: HashSet::new(),
            rmod: rmod,
            analyzer: T::new(),
        }
    }

    fn analyze_function(&mut self, func_addr: u64) {
        // If the current function has already been analyzed, return.
        if self.analyzed.contains(&func_addr) {
            return;
        }
        // Analyze all children of the present node in call graph.
        let callees = self.rmod.function(func_addr).map(|rfn| {
            self.rmod.callees_of(rfn)
        }).unwrap_or(Vec::new());

        for (call, _) in callees {
            self.analyze_function(call);
        }

        // Propagate changes and remove deadcode based on the analysis information from
        // the children. Perform context translations from caller to callee etc.
        // TODO.
        {
            // Pull changes from callee.
            self.analyzer.propagate(self.rmod, func_addr);
            // Analyze transfer function for the current function.
            self.analyzer.transfer(self.rmod, func_addr);
        }

        // Insert the current function into analyzed set.
        self.analyzed.insert(func_addr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use frontend::source::FileSource;
    // use frontend::source::Source;
    use frontend::containers::*;
    use middle::ir_writer::IRWriter;
    use middle::dce;
    use analysis::interproc::summary;
    // use r2pipe::r2::R2;

    #[test]
    #[ignore]
    fn ipa_t1() {
        //let mut r2 = R2::new(Some("./ct1_sccp_ex.o")).expect("Failed to open r2");
        //r2.init();
        //let mut fsource = FileSource::from(r2);
        let mut fsource = FileSource::open(Some("./test_files/ct1_sccp_ex/ct1_sccp_ex"));
        let mut rmod = RadecoModule::from(&mut fsource);
        {
            analyze_module::<_, summary::CallSummary>(&mut rmod);
        }

        for (ref addr, ref mut rfn) in rmod.functions.iter_mut() {
            {
                dce::collect(&mut rfn.ssa);
            }
            //println!("Binds: {:?}", rfn.bindings.bindings());
            println!("Info for: {:#x}", addr);
            println!("Local Variable info: {:#?}", rfn.locals());
            println!("Arg info: {:#?}", rfn.args());
            //println!("Returns info: {:?}", rfn.returns());
            let mut writer: IRWriter = Default::default();
            println!("{}", writer.emit_il(Some(rfn.name.clone()), &rfn.ssa));
        }
    }
}
