use crate::ast::*;
use crate::resolve::gensym;
use std::mem;

pub fn run(fields: &mut Vec<ModuleField>) {
    let mut cur = 0;
    let mut to_append = Vec::new();
    while cur < fields.len() {
        let item = &mut fields[cur];
        match item {
            ModuleField::Func(f) => {
                for name in f.exports.names.drain(..) {
                    let id = gensym::fill(f.span, &mut f.id);
                    to_append.push(ModuleField::Export(Export {
                        span: f.span,
                        name,
                        kind: ExportKind::Func(Index::Id(id)),
                    }));
                }
                match f.kind {
                    FuncKind::Import(import) => {
                        *item = ModuleField::Import(Import {
                            span: f.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: f.span,
                                id: f.id,
                                name: f.name,
                                kind: ItemKind::Func(f.ty.clone()),
                            },
                        });
                    }
                    FuncKind::Inline { .. } => {}
                }
            }

            ModuleField::Memory(m) => {
                for name in m.exports.names.drain(..) {
                    let id = gensym::fill(m.span, &mut m.id);
                    to_append.push(ModuleField::Export(Export {
                        span: m.span,
                        name,
                        kind: ExportKind::Memory(Index::Id(id)),
                    }));
                }
                match m.kind {
                    MemoryKind::Import { import, ty } => {
                        *item = ModuleField::Import(Import {
                            span: m.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: m.span,
                                id: m.id,
                                name: None,
                                kind: ItemKind::Memory(ty),
                            },
                        });
                    }
                    // If data is defined inline insert an explicit `data` module
                    // field here instead, switching this to a `Normal` memory.
                    MemoryKind::Inline { is_32, ref data } => {
                        let len = data.iter().map(|l| l.len()).sum::<usize>() as u32;
                        let pages = (len + page_size() - 1) / page_size();
                        let kind = MemoryKind::Normal(if is_32 {
                            MemoryType::B32 {
                                limits: Limits {
                                    min: pages,
                                    max: Some(pages),
                                },
                                shared: false,
                                secret: false,
                            }
                        } else {
                            MemoryType::B64 {
                                limits: Limits64 {
                                    min: u64::from(pages),
                                    max: Some(u64::from(pages)),
                                },
                                shared: false,
                                secret: false,
                            }
                        });
                        let data = match mem::replace(&mut m.kind, kind) {
                            MemoryKind::Inline { data, .. } => data,
                            _ => unreachable!(),
                        };
                        let id = gensym::fill(m.span, &mut m.id);
                        to_append.push(ModuleField::Data(Data {
                            span: m.span,
                            id: None,
                            kind: DataKind::Active {
                                memory: Index::Id(id),
                                offset: Expression {
                                    instrs: Box::new([Instruction::I32Const(0)]),
                                },
                            },
                            data,
                        }));
                    }

                    MemoryKind::Normal(_) => {}
                }
            }

            ModuleField::Table(t) => {
                for name in t.exports.names.drain(..) {
                    let id = gensym::fill(t.span, &mut t.id);
                    to_append.push(ModuleField::Export(Export {
                        span: t.span,
                        name,
                        kind: ExportKind::Table(Index::Id(id)),
                    }));
                }
                match &mut t.kind {
                    TableKind::Import { import, ty } => {
                        *item = ModuleField::Import(Import {
                            span: t.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: t.span,
                                id: t.id,
                                name: None,
                                kind: ItemKind::Table(*ty),
                            },
                        });
                    }
                    // If data is defined inline insert an explicit `data` module
                    // field here instead, switching this to a `Normal` memory.
                    TableKind::Inline { payload, elem } => {
                        let len = match payload {
                            ElemPayload::Indices(v) => v.len(),
                            ElemPayload::Exprs { exprs, .. } => exprs.len(),
                        };
                        let kind = TableKind::Normal(TableType {
                            limits: Limits {
                                min: len as u32,
                                max: Some(len as u32),
                            },
                            elem: *elem,
                        });
                        let payload = match mem::replace(&mut t.kind, kind) {
                            TableKind::Inline { payload, .. } => payload,
                            _ => unreachable!(),
                        };
                        let id = gensym::fill(t.span, &mut t.id);
                        to_append.push(ModuleField::Elem(Elem {
                            span: t.span,
                            id: None,
                            kind: ElemKind::Active {
                                table: Index::Id(id),
                                offset: Expression {
                                    instrs: Box::new([Instruction::I32Const(0)]),
                                },
                            },
                            payload,
                        }));
                    }

                    TableKind::Normal(_) => {}
                }
            }

            ModuleField::Global(g) => {
                for name in g.exports.names.drain(..) {
                    let id = gensym::fill(g.span, &mut g.id);
                    to_append.push(ModuleField::Export(Export {
                        span: g.span,
                        name,
                        kind: ExportKind::Global(Index::Id(id)),
                    }));
                }
                match g.kind {
                    GlobalKind::Import(import) => {
                        *item = ModuleField::Import(Import {
                            span: g.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: g.span,
                                id: g.id,
                                name: None,
                                kind: ItemKind::Global(g.ty),
                            },
                        });
                    }
                    GlobalKind::Inline { .. } => {}
                }
            }

            ModuleField::Event(e) => {
                for name in e.exports.names.drain(..) {
                    let id = gensym::fill(e.span, &mut e.id);
                    to_append.push(ModuleField::Export(Export {
                        span: e.span,
                        name,
                        kind: ExportKind::Event(Index::Id(id)),
                    }));
                }
            }

            ModuleField::Instance(i) => {
                for name in i.exports.names.drain(..) {
                    let id = gensym::fill(i.span, &mut i.id);
                    to_append.push(ModuleField::Export(Export {
                        span: i.span,
                        name,
                        kind: ExportKind::Instance(Index::Id(id)),
                    }));
                }
                match &mut i.kind {
                    InstanceKind::Import { import, ty } => {
                        *item = ModuleField::Import(Import {
                            span: i.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: i.span,
                                id: i.id,
                                name: None,
                                kind: ItemKind::Instance(mem::replace(
                                    ty,
                                    TypeUse::new_with_index(Index::Num(0, Span::from_offset(0))),
                                )),
                            },
                        });
                    }
                    InstanceKind::Inline { .. } => {}
                }
            }

            ModuleField::NestedModule(m) => {
                for name in m.exports.names.drain(..) {
                    let id = gensym::fill(m.span, &mut m.id);
                    to_append.push(ModuleField::Export(Export {
                        span: m.span,
                        name,
                        kind: ExportKind::Module(Index::Id(id)),
                    }));
                }
                match &mut m.kind {
                    NestedModuleKind::Import { import, ty } => {
                        *item = ModuleField::Import(Import {
                            span: m.span,
                            module: import.module,
                            field: import.field,
                            item: ItemSig {
                                span: m.span,
                                id: m.id,
                                name: m.name,
                                kind: ItemKind::Module(mem::replace(
                                    ty,
                                    TypeUse::new_with_index(Index::Num(0, Span::from_offset(0))),
                                )),
                            },
                        });
                    }
                    NestedModuleKind::Inline { fields, .. } => {
                        run(fields);
                    }
                };
            }

            ModuleField::ExportAll(..)
            | ModuleField::Import(_)
            | ModuleField::Type(_)
            | ModuleField::Export(_)
            | ModuleField::Alias(_)
            | ModuleField::Start(_)
            | ModuleField::Elem(_)
            | ModuleField::Data(_)
            | ModuleField::Custom(_) => {}
        }

        fields.splice(cur..cur, to_append.drain(..));
        cur += 1;
    }

    assert!(to_append.is_empty());

    fn page_size() -> u32 {
        1 << 16
    }
}
