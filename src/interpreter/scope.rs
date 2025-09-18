use std::collections::HashMap;
use std::mem;
use rustc_hash::FxHashMap;
use crate::interpreter::Primitive;
use crate::ast::*;


/// スコープを管理します。
#[derive(Default)]
pub struct Scope(Vec<FxHashMap<Ident, Primitive>>);

impl Scope {
	pub fn new() -> Self {
		Self(vec![FxHashMap::default()])
	}
	/// 新しいスコープを生成します。
	pub fn create(&mut self) {
		self.0.push(FxHashMap::default());
	}
	/// 最上位のスコープを破棄します。
	pub fn delete(&mut self) {
		self.0.pop().unwrap();
	}
	/// 変数を追加します。
	pub fn define(&mut self, ident: Ident, value: Primitive) {
		self.0
		.last_mut()
		.unwrap()
		.insert(ident, value);
	}
	/// 変数名からスコープを検索して値を返します。
	pub fn get(&self, ident: &Ident) -> Option<Primitive> {
		for scope in self.0.iter().rev() {
			let value = scope.get(ident);
			if value.is_some() {
				return value.cloned()
			}
		}
		None
	}
	/// 変数に代入します。
	pub fn assign(&mut self, ident: &Ident, value: Primitive) -> Result<(), ()> {
		for scope in self.0.iter_mut().rev() {
			if let Some(var) = scope.get_mut(ident) {
				if mem::discriminant(var) == mem::discriminant(&value) {
					*var = value;
					return Ok(());
				} else {
					return Err(());
				}
			}
		}
		Err(())
	}
	/// グローバルスコープを取得します。
	pub fn create_global(&mut self) -> Global {
		let mut others = mem::take(self).0.into_iter().rev().collect::<Vec<_>>();
		let global = Scope(vec![others.pop().unwrap()]);
		Global { global, others }
	}
}

pub struct Global {
	global: Scope,
	others: Vec<FxHashMap<Ident, Primitive>>,
}

impl Global {
	/// グローバルスコープへの可変参照を取得します。
	pub fn get_mut(&mut self) -> &mut Scope {
		&mut self.global
	}
	/// グローバルとそれ以外に分割されたScopeから元のScopeに戻します。
	pub fn reconstruct(mut self, scope: &mut Scope) {
		self.others.into_iter().for_each(|map|self.global.0.push(map));
		*scope = self.global;
	}
}