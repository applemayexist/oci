project(
    'sd-spawn',
    'c',
    'rust',
    default_options: {
        'warning_level': '3',
        'b_ndebug': 'if-release',
        'rust_std': '2024',
    },
    meson_version: '>=1.4.0',
    version: '1~devel',
)

rust = import('rust')

conf = configuration_data()
conf.set_quoted('SYSTEMD_RUN_PATH', find_program('systemd-run').full_path())

executable(
    'sd-spawn',
    structured_sources(
        [
            'sd-spawn.rs',
            configure_file(
                configuration: conf,
                input: 'config.rs.in',
                output: '@BASENAME@',
            ),
            rust.bindgen(c_args: '-D_GNU_SOURCE', input: 'headers.h', output: 'sys.rs'),
        ],
    ),
    install: true,
    override_options: {'rust_std': '2024'},
)
