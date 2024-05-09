// Copyright 2019 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use protobuf_build::Builder;
use std::fs;

fn main() {
    let proto_dir = "proto";
    let mut proto_files = Vec::new();

    // Walk the proto directory and collect all .proto files.
    for entry in fs::read_dir(proto_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "proto" {
                    proto_files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    let mut builder = Builder::new();
    customize_protobuf_gen(&mut builder);
    builder
        .files(&proto_files)
        .append_to_black_list("eraftpb")
        .generate()
}

fn customize_protobuf_gen(builder: &mut Builder) {
    use protobuf::descriptor::DescriptorProto;
    use protobuf::descriptor::EnumDescriptorProto;
    use protobuf::descriptor::FieldDescriptorProto;
    use protobuf::descriptor::FieldDescriptorProto_Label;
    use protobuf::descriptor::FileDescriptorProto;
    use protobuf::descriptor::OneofDescriptorProto;
    use protobuf_codegen::{Customize, CustomizeCallback};

    struct HeapSizeCustomizeCallback;
    impl CustomizeCallback for HeapSizeCustomizeCallback {
        fn message(&self, _: &DescriptorProto) -> Customize {
            Customize::default().before("#[derive(::heapsz_derive::HeapSize)]\n#[heap_size]")
        }

        fn field(&self, field: &FieldDescriptorProto) -> Customize {
            let with = match field.get_type_name() {
                ".eraftpb.ConfChangeType" => "crate::heap_size_eraftpb::conf_change_type_field",
                ".eraftpb.Entry" => {
                    if field.get_label() == FieldDescriptorProto_Label::LABEL_REPEATED {
                        "crate::heap_size_eraftpb::entry_repeated_field"
                    } else {
                        "crate::heap_size_eraftpb::entry_optional_field"
                    }
                }
                ".eraftpb.Message" => "crate::heap_size_eraftpb::message_field",
                ".eraftpb.HardState" => "crate::heap_size_eraftpb::hard_state_field",
                _ => "",
            };
            if with.is_empty() {
                Customize::default()
            } else {
                Customize::default().before(&format!("#[heap_size(with=\"{with}\")]"))
            }
        }

        fn special_field(&self, _: &DescriptorProto, _: &str) -> Customize {
            Customize::default().before("#[heap_size(skip)]")
        }

        fn enumeration(&self, _: &EnumDescriptorProto) -> Customize {
            Customize::default().before("#[derive(::heapsz_derive::HeapSize)]\n#[heap_size]")
        }

        fn oneof(&self, _: &OneofDescriptorProto) -> Customize {
            Customize::default().before("#[derive(::heapsz_derive::HeapSize)]\n#[heap_size]")
        }
    }

    builder.customize_protobuf_gen(Box::new(
        move |file_descriptors: &[FileDescriptorProto],
              files_to_generate: &[String],
              customize: &Customize| {
            protobuf_codegen::gen_with_callback(
                file_descriptors,
                files_to_generate,
                customize,
                &HeapSizeCustomizeCallback,
            )
        },
    ));
}
