use tantivy::{schema::*, Index, doc, collector::TopDocs, query::QueryParser};
pub fn search(q: &str) -> usize {
    let mut sb = Schema::builder();
    let title = sb.add_text_field("title", TEXT | STORED);
    let index = Index::create_in_ram(sb.build());
    let mut w = index.writer(15_000_000).unwrap();
    for i in 0..100 { w.add_document(doc!(title => format!("payments service {i}"))).unwrap(); }
    w.commit().unwrap();
    let r = index.reader().unwrap();
    let qp = QueryParser::for_index(&index, vec![title]);
    r.searcher().search(&qp.parse_query(q).unwrap(), &TopDocs::with_limit(10).order_by_score()).unwrap().len()
}
