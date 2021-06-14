mod fixtures;

use evtx::{EvtxParser, ParserSettings, EvtxStructureVisitor};
use fixtures::*;
use log::Level;
use std::path::Path;

/// Tests an .evtx file, asserting the number of parsed records matches `count`.
fn test_full_sample(path: impl AsRef<Path>, ok_count: usize, err_count: usize) {
    ensure_env_logger_initialized();
    let mut parser = EvtxParser::from_path(path).unwrap();

    let mut actual_ok_count = 0;
    let mut actual_err_count = 0;

    for r in parser.records() {
        if r.is_ok() {
            actual_ok_count += 1;
            if log::log_enabled!(Level::Debug) {
                println!("{}", r.unwrap().data);
            }
        } else {
            actual_err_count += 1;
        }
    }
    assert_eq!(
        actual_ok_count, ok_count,
        "XML: Failed to parse all expected records"
    );
    assert_eq!(actual_err_count, err_count, "XML: Expected errors");

    let mut actual_ok_count = 0;
    let mut actual_err_count = 0;

    for r in parser.records_json() {
        if r.is_ok() {
            actual_ok_count += 1;
            if log::log_enabled!(Level::Debug) {
                println!("{}", r.unwrap().data);
            }
        } else {
            actual_err_count += 1;
        }
    }
    assert_eq!(
        actual_ok_count, ok_count,
        "Failed to parse all records as JSON"
    );
    assert_eq!(actual_err_count, err_count, "XML: Expected errors");

    let mut actual_ok_count = 0;
    let mut actual_err_count = 0;
    let seperate_json_attributes = ParserSettings::default().separate_json_attributes(true);
    parser = parser.with_configuration(seperate_json_attributes);

    for r in parser.records_json() {
        if r.is_ok() {
            actual_ok_count += 1;
            if log::log_enabled!(Level::Debug) {
                println!("{}", r.unwrap().data);
            }
        } else {
            actual_err_count += 1;
        }
    }
    assert_eq!(
        actual_ok_count, ok_count,
        "Failed to parse all records as JSON"
    );
    assert_eq!(actual_err_count, err_count, "XML: Expected errors");

    let mut actual_ok_count = 0;
    let mut actual_err_count = 0;

    for r in parser.records_to_visitor(|| TestVisitor{}) {
      if r.is_ok() {
          actual_ok_count += 1;
          if log::log_enabled!(Level::Debug) {
              println!("error");
          }
      } else {
          actual_err_count += 1;
      }
    }
    
    assert_eq!(
      actual_ok_count, ok_count,
      "XML: Failed to parse all expected records"
    );
    assert_eq!(actual_err_count, err_count, "XML: Expected errors");
}

#[test]
// https://github.com/omerbenamram/evtx/issues/10
fn test_dirty_sample_single_threaded() {
    ensure_env_logger_initialized();
    let evtx_file = include_bytes!("../samples/2-system-Security-dirty.evtx");

    let mut parser = EvtxParser::from_buffer(evtx_file.to_vec()).unwrap();

    let mut count = 0;
    for r in parser.records() {
        r.unwrap();
        count += 1;
    }
    assert_eq!(count, 14621, "Single threaded iteration failed");
}

#[test]
fn test_dirty_sample_parallel() {
    ensure_env_logger_initialized();
    let evtx_file = include_bytes!("../samples/2-system-Security-dirty.evtx");

    let mut parser = EvtxParser::from_buffer(evtx_file.to_vec())
        .unwrap()
        .with_configuration(ParserSettings::new().num_threads(8));

    let mut count = 0;

    for r in parser.records() {
        r.unwrap();
        count += 1;
    }

    assert_eq!(count, 14621, "Parallel iteration failed");
}

#[test]
fn test_parses_sample_with_irregular_boolean_values() {
    test_full_sample(sample_with_irregular_values(), 3028, 0);
}

#[test]
fn test_dirty_sample_with_a_bad_checksum() {
    test_full_sample(sample_with_a_bad_checksum(), 1910, 4)
}

#[test]
fn test_dirty_sample_with_a_bad_checksum_2() {
    // TODO: investigate 2 failing records
    test_full_sample(sample_with_a_bad_checksum_2(), 1774, 2)
}

#[test]
fn test_dirty_sample_with_a_chunk_past_zeros() {
    test_full_sample(sample_with_a_chunk_past_zeroes(), 1160, 0)
}

#[test]
fn test_dirty_sample_with_a_bad_chunk_magic() {
    test_full_sample(sample_with_a_bad_chunk_magic(), 270, 5)
}

#[test]
fn test_dirty_sample_binxml_with_incomplete_token() {
    // Contains an unparsable record
    test_full_sample(sample_binxml_with_incomplete_sid(), 6, 1)
}

#[test]
fn test_dirty_sample_binxml_with_incomplete_template() {
    // Contains an unparsable record
    test_full_sample(sample_binxml_with_incomplete_template(), 17, 1)
}

#[test]
fn test_sample_with_multiple_xml_fragments() {
    test_full_sample(sample_with_multiple_xml_fragments(), 1146, 0)
}

#[test]
fn test_issue_65() {
    test_full_sample(sample_issue_65(), 459, 0)
}

#[test]
fn test_sample_with_binxml_as_substitution_tokens_and_pi_target() {
    test_full_sample(
        sample_with_binxml_as_substitution_tokens_and_pi_target(),
        340,
        0,
    )
}

#[test]
fn test_sample_with_dependency_identifier_edge_case() {
    test_full_sample(sample_with_dependency_id_edge_case(), 653, 0)
}

#[test]
fn test_sample_with_no_crc32() {
    test_full_sample(
        sample_with_no_crc32(),
        17,
        0,
    )
}

struct TestVisitor {}
impl EvtxStructureVisitor for TestVisitor {
  type VisitorResult = Option<()>;

  fn get_result(&self, _event_record_id: u64, _timestamp: chrono::DateTime<chrono::Utc>) -> Self::VisitorResult {
      Some(())
  }

  /// called when a new record starts
  fn start_record(&mut self) {}

  /// called when the current records is finished
  fn finalize_record(&mut self) {}

  // called upon element content
  fn visit_characters(&mut self, _value: &str) {}

  /// called on any structure element with a content type of `None`
  fn visit_empty_element<'a, 'b>(&'a mut self, _name: &'b str, _attributes: Box<dyn Iterator<Item=(&'b str, &'b str)> + 'b>) where 'a: 'b {}

  /// called on any structure element which contains only a textual value
  fn visit_simple_element<'a, 'b>(&'a mut self, _name: &'b str, _attributes: Box<dyn Iterator<Item=(&'b str, &'b str)> + 'b>, _content: &'b str) where 'a: 'b {}

  /// called when a complex element (i.e. an element with child elements) starts
  fn visit_start_element<'a, 'b>(&'a mut self, _name: &'b str, _attributes: Box<dyn Iterator<Item=(&'b str, &'b str)> + 'b>) where 'a: 'b {}

  /// called when a complex element (i.e. an element with child elements) ends
  fn visit_end_element(&mut self, _name: &str) {}
}