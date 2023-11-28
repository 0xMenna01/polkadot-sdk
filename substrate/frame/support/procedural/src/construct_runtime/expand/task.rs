// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License

use crate::construct_runtime::Pallet;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

/// Expands aggregate `RuntimeTask` enum.
pub fn expand_outer_task(
	runtime_name: &Ident,
	pallet_decls: &[Pallet],
	scrate: &TokenStream2,
) -> TokenStream2 {
	let mut from_impls = Vec::new();
	let mut task_variants = Vec::new();
	let mut variant_names = Vec::new();
	let mut task_paths = Vec::new();
	for decl in pallet_decls {
		if let Some(_) = decl.find_part("Task") {
			let variant_name = &decl.name;
			let path = &decl.path;
			let index = decl.index;

			from_impls.push(quote! {
				impl From<#path::Task<#runtime_name>> for RuntimeTask {
					fn from(hr: #path::Task<#runtime_name>) -> Self {
						RuntimeTask::#variant_name(hr)
					}
				}

				impl From<RuntimeTask> for Option<#path::Task<#runtime_name>> {
					fn from(rt: RuntimeTask) -> Self {
						match rt {
							RuntimeTask::#variant_name(hr) => Some(hr),
							_ => None,
						}
					}
				}
			});

			task_variants.push(quote! {
				#[codec(index = #index)]
				#variant_name(#path::Task<#runtime_name>),
			});

			variant_names.push(quote!(#variant_name));

			task_paths.push(quote!(#path::Task));
		}
	}

	let prelude = quote!(#scrate::traits::tasks::prelude);

	let output = quote! {
		/// An aggregation of all `Task` enums across all pallets included in the current runtime.
		#[derive(
			Clone, Eq, PartialEq,
			#scrate::__private::codec::Encode, 
			#scrate::__private::codec::Decode,
			#scrate::__private::scale_info::TypeInfo,
			#scrate::__private::RuntimeDebug,
		)]
		pub enum RuntimeTask {
			#( #task_variants )*
		}

		#[automatically_derived]
		impl #scrate::traits::Task for RuntimeTask {
			type Enumeration = #prelude::IntoIter<RuntimeTask>;

			fn is_valid(&self) -> bool {
				match self {
					#(RuntimeTask::#variant_names(val) => val.is_valid(),)*
				}
			}

			fn run(&self) -> Result<(), #scrate::traits::tasks::prelude::DispatchError> {
				match self {
					#(RuntimeTask::#variant_names(val) => val.run(),)*
				}
			}

			fn weight(&self) -> #scrate::pallet_prelude::Weight {
				match self {
					#(RuntimeTask::#variant_names(val) => val.weight(),)*
				}
			}

			fn task_index(&self) -> u32 {
				match self {
					#(RuntimeTask::#variant_names(val) => val.task_index(),)*
				}
			}

			fn iter() -> Self::Enumeration {
				let mut all_tasks = Vec::new();
				#(all_tasks.extend(#task_paths::iter().map(RuntimeTask::from).collect::<Vec<_>>());)*
				all_tasks.into_iter()
			}
		}

		#( #from_impls )*
	};

	output
}
