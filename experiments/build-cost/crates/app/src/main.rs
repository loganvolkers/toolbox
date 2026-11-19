// EDIT-MARKER 0
fn main() {
    println!("{:?}", model::entity(1).id);
    #[cfg(feature = "boring")]
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let _r = leaf_web::router();
        let _p = leaf_store::pool();
        println!("mcp {}", leaf_mcp::info().len());
        println!("cel {}", leaf_cel::eval("tier <= 2", 1));
    });
    #[cfg(feature = "full")]
    {
        println!("closure {}", leaf_ivm::closure_size(&[(1, 2), (2, 3), (3, 4)]));
        println!("wasm {}", leaf_wasm::add(2, 3));
        println!("search {}", leaf_search::search("payments"));
        println!("graphql {}", leaf_graphql::schema_sdl(&["Service", "Group"]).len());
    }
}
