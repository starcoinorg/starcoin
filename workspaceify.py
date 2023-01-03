from abc import abstractmethod

import toml
import argparse


def merge_two_dicts(x, y):
    z = x.copy()  # start with keys and values of x
    z.update(y)  # modifies z with keys and values of y
    return z


def normalized_relative_path(work_dir, input_path):
    """
    :param self:
    :param work_dir:
    :param input_path:
    :return: new path relative workdir
    """
    return input_path


class TomlObj:
    def __init__(self, file_path):
        self.toml_dict = dict()
        self.file_path = file_path

    def load(self):
        self.toml_dict = toml.load(self.file_path)

    def save(self):
        with open(self.file_path, mode="w") as f:
            toml.dump(self.toml_dict, f, encoder=toml.TomlPreserveInlineDictEncoder())

    def get_origin_dependencies(self, work_dir, is_dev=False):
        depends = self.get_dependencies(is_dev)
        origin_depends = dict()
        # Ignore dependencies item with workspace
        for i, (k, v) in enumerate(depends.items()):
            if isinstance(v, dict) and 'workspace' in v:
                continue

            if k == 'path':
                origin_depends['path'] = normalized_relative_path(work_dir, v)
                continue

            origin_depends[k] = v
        return origin_depends

    @abstractmethod
    def get_dependencies(self, is_dev=False):
        pass

    @abstractmethod
    def put_dependencies(self, depends):
        pass

    def combine_dependencies(self, input_depends):
        if len(input_depends) <= 0:
            return

        dependencies = self.get_dependencies()
        new_dependencies = merge_two_dicts(dependencies, input_depends)
        self.put_dependencies(new_dependencies)

    def replace_package(self):
        """
        authors = { workspace = true }
        edition = { workspace = true }
        homepage = { workspace = true }
        license = { workspace = true }
        publish = { workspace = true }
        repository = { workspace = true }
        rust-version = { workspace = true }
        :return:
        """
        decoder = toml.TomlDecoder()
        pacakge = self.toml_dict['package']

        authors = decoder.get_empty_inline_table()
        authors['workspace'] = True
        pacakge['authors'] = authors

        edition = decoder.get_empty_inline_table()
        edition['workspace'] = True
        pacakge['edition'] = edition

        homepage = decoder.get_empty_inline_table()
        homepage['workspace'] = True
        pacakge['homepage'] = homepage

        license = decoder.get_empty_inline_table()
        license['workspace'] = True
        pacakge['license'] = license

        publish = decoder.get_empty_inline_table()
        publish['workspace'] = True
        pacakge['publish'] = publish

        repository = decoder.get_empty_inline_table()
        repository['workspace'] = True
        pacakge['repository'] = repository

        rustversion = decoder.get_empty_inline_table()
        rustversion['workspace'] = True
        pacakge['rust-version'] = rustversion

    def replace_dependencies(self, is_dev=False):
        depends = self.get_dependencies(is_dev)
        decoder = toml.TomlDecoder()
        dict_key = ['path', 'version', 'git', 'rev']
        for i, (k, v) in enumerate(depends.items()):
            if isinstance(v, dict):
                if 'workspace' in v:
                    continue

                # Remove key from dicts
                for dk in dict_key:
                    if dk in v:
                        del v[dk]

                v['workspace'] = True
            else:
                inline_table = decoder.get_empty_inline_table()
                inline_table['workspace'] = True
                depends[k] = inline_table


class RootCargoToml(TomlObj):
    def __init__(self, file_path):
        super().__init__(file_path)

    def get_dependencies(self, is_dev=False):
        return self.toml_dict['workspace']['dependencies']

    def put_dependencies(self, depends):
        self.toml_dict['workspace']['dependencies'] = depends


class SubCargoToml(TomlObj):
    def __init__(self, file_path):
        super().__init__(file_path)

    def get_dependencies(self, is_dev=False):
        return self.toml_dict['dependencies' if is_dev is False else 'dev-dependencies']

    def put_dependencies(self, depends):
        self.toml_dict['dependencies'] = depends


def main(work_dir):
    root_toml = RootCargoToml(work_dir + "/Cargo.toml")
    root_toml.load()

    test_toml = SubCargoToml(work_dir + "/consensus/Cargo.toml")
    test_toml.load()
    root_toml.combine_dependencies(test_toml.get_origin_dependencies(work_dir, is_dev=False))
    root_toml.combine_dependencies(test_toml.get_origin_dependencies(work_dir, is_dev=True))
    test_toml.replace_package()
    test_toml.replace_dependencies(is_dev=False)
    test_toml.replace_dependencies(is_dev=True)
    test_toml.save()

    # Save root toml
    root_toml.save()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Replace Cargo.toml to workspace method")
    parser.add_argument('--work-dir', required=True, dest='workdir', help='Root work directory')

    args = parser.parse_args()

    main(work_dir=args.workdir)
