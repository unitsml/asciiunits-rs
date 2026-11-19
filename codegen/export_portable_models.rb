# frozen_string_literal: true

# Exports the AsciiUnits Ruby model (Asciiunits::Model) as a portable LML
# bundle for lutaml-rs. Consumes lutaml-model's bundle API; does not implement
# a codec. The calling environment pins lutaml-model and the asciiunits source.
#
# Usage: ruby codegen/export_portable_models.rb ASCIIUNITS_CHECKOUT OUTPUT
require "json"

abort "usage: ruby codegen/export_portable_models.rb ASCIIUNITS_CHECKOUT OUTPUT" unless ARGV.length == 2
source, output = ARGV
$LOAD_PATH.unshift(File.expand_path("lib", source))
require "lutaml/model"
require "lutaml/model/lml"
require "asciiunits/model"

abort "portable LML bundle API is unavailable in this lutaml-model" unless Lutaml::Model::Lml.respond_to?(:export_bundle)

# Ruby supplies implicit JSON attribute mappings. Declare that same mapping
# vocabulary in this isolated export process so foreign generators can see it;
# ordinary Ruby consumers keep their existing writer configuration.
Asciiunits::Model.classes.each do |klass|
  names = klass.attributes.keys
  klass.json do
    names.each { |name| map name.to_s, to: name }
  end
end
bundle = Lutaml::Model::Lml.export_bundle([Asciiunits::Model::Document])
File.write(output, "#{JSON.pretty_generate(bundle.to_h)}\n")
puts "exported #{bundle.to_h.fetch('models').size} models to #{output}"
