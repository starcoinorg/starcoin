import collections
import copy
from abc import abstractmethod

import toml
import argparse


def merge_two_dicts(x, y):
    z = x.copy()  # start with keys and values of x
    z.update(y)  # modifies z with keys and values of y
    return z


def normalized_relative_path(input_path):
    """
    :param input_path:
    :return: new path relative workdir
    """

    return input_path.replace('../', '')


class TomlObj:
    def __init__(self, work_dir, file_path):
        self.toml_dict = dict()
        self.file_path = file_path
        self.work_dir = work_dir

    def load(self):
        self.toml_dict = toml.load(self.file_path, decoder=toml.decoder.TomlDecoder())

    def save(self):
        with open(self.file_path, mode="w") as f:
            toml.dump(collections.OrderedDict(sorted(self.toml_dict.items(), key=lambda item: item[0])),
                      f,
                      encoder=toml.TomlPreserveInlineDictEncoder())
        # print("Save filepath: " + self.file_path)
        # print(toml.dumps(self.toml_dict, encoder=toml.TomlPreserveInlineDictEncoder()))

    def get_item(self, key):
        return self.toml_dict[key]

    def get_origin_dependencies(self, is_dev=False):
        depends = self.get_dependencies(is_dev)
        origin_depends = dict()
        if depends is None:
            return dict()

        # Ignore dependencies item with workspace
        for i, (k, v) in enumerate(depends.items()):
            new_v = v
            if isinstance(new_v, dict):
                if 'workspace' in new_v:
                    continue
                if 'path' in new_v:
                    new_v['path'] = normalized_relative_path(new_v['path'])
            origin_depends[k] = new_v
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
        if dependencies is None:
            return

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
        package = self.toml_dict['package']

        authors = decoder.get_empty_inline_table()
        authors['workspace'] = True
        package['authors'] = authors

        edition = decoder.get_empty_inline_table()
        edition['workspace'] = True
        package['edition'] = edition

        homepage = decoder.get_empty_inline_table()
        homepage['workspace'] = True
        package['homepage'] = homepage

        license = decoder.get_empty_inline_table()
        license['workspace'] = True
        package['license'] = license

        publish = decoder.get_empty_inline_table()
        publish['workspace'] = True
        package['publish'] = publish

        repository = decoder.get_empty_inline_table()
        repository['workspace'] = True
        package['repository'] = repository

        rustversion = decoder.get_empty_inline_table()
        rustversion['workspace'] = True
        package['rust-version'] = rustversion

    def replace_dependencies(self, is_dev=False):
        depends = self.get_dependencies(is_dev)
        if depends is None:
            return

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

    @abstractmethod
    def replace_fake_package_name(self):
        pass


class RootCargoToml(TomlObj):
    def __init__(self, work_dir, file_path):
        super().__init__(work_dir, file_path)

    def get_dependencies(self, is_dev=False):
        if 'workspace' not in self.toml_dict:
            self.toml_dict['workspace'] = dict()

        if 'dependencies' not in self.toml_dict['workspace']:
            self.toml_dict['workspace'] = dict()

        return self.toml_dict['workspace']['dependencies']

    def put_dependencies(self, depends):
        self.toml_dict['workspace']['dependencies'] = copy.deepcopy(depends)

    def replace_fake_package_name(self):
        depends = self.get_dependencies()
        remove_keys = []
        for i, (k, v) in enumerate(depends.items()):
            if isinstance(v, dict) and 'path' in v:
                path = v['path']
                subtoml = SubCargoToml(self.work_dir,
                                       self.work_dir + path + "/Cargo.toml")
                subtoml.load()
                real_name = subtoml.get_item('package')['name']
                if real_name != k and real_name in depends:
                    remove_keys.append(k)

        for k in remove_keys:
            del depends[k]


class SubCargoToml(TomlObj):
    def __init__(self, work_dir, file_path):
        super().__init__(work_dir, file_path)

    def get_dependencies(self, is_dev=False):
        key = 'dependencies' if is_dev is False else 'dev-dependencies'
        return self.toml_dict[key] if key in self.toml_dict else None

    def put_dependencies(self, depends):
        self.toml_dict['dependencies'] = copy.deepcopy(depends)

    def replace_fake_package_name(self):
        decoder = toml.decoder.TomlDecoder()
        add_dicts = dict()
        remove_keys = []
        depends = self.get_dependencies()
        for i, (k, v) in enumerate(depends.items()):
            if isinstance(v, dict) and 'package' in v:
                new_key = v['package']
                if k == new_key:
                    continue

                inline_table = decoder.get_empty_inline_table()
                inline_table['workspace'] = True
                add_dicts[new_key] = inline_table
                remove_keys.append(k)
        for rk in remove_keys:
            del depends[rk]


def replace_fake_package_names_sub_proj(root_toml, work_dir):
    root_workspace = root_toml.get_item('workspace')
    members = root_workspace["members"]
    for m in members:
        sub_toml = SubCargoToml(work_dir, work_dir + m + "/Cargo.toml")
        sub_toml.load()
        sub_toml.replace_fake_package_name()
        sub_toml.save()


def main(work_dir):
    root_toml = RootCargoToml(work_dir, work_dir + "Cargo.toml")
    root_toml.load()

    replace_fake_package_names_sub_proj(root_toml, work_dir)

    # root_workspace = root_toml.get_item('workspace')
    # members = root_workspace["members"]
    # for m in members:
    #     sub_cargo_item = work_dir + m + "/Cargo.toml"
    #     test_toml = SubCargoToml(sub_cargo_item)
    #     test_toml.load()
    #
    #     release_dep = test_toml.get_origin_dependencies(is_dev=False)
    #     root_toml.combine_dependencies(release_dep)
    #     debug_dep = test_toml.get_origin_dependencies(is_dev=True)
    #     root_toml.combine_dependencies(debug_dep)
    #
    #     test_toml.replace_package()
    #     test_toml.replace_dependencies(is_dev=False)
    #     test_toml.replace_dependencies(is_dev=True)
    #     test_toml.save()

    # Save root toml
    # sorted_dict = collections.OrderedDict(sorted(root_workspace['dependencies'].items(), key=lambda item: item[0]))
    # trim_repeatly_items(sorted_dict, work_dir)
    # root_workspace['dependencies'] = sorted_dict

    # root_toml.save()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Replace Cargo.toml to workspace method")
    parser.add_argument('--work-dir', required=True, dest='workdir', help='Root work directory')

    args = parser.parse_args()

    main(work_dir=args.workdir)
