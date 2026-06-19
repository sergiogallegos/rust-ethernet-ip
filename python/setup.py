"""Setuptools shim.

The package metadata lives in ``pyproject.toml``. This shim exists only to mark
the wheel as platform-specific: the package bundles a prebuilt native library
(``.so``/``.dll``/``.dylib``) staged by ``scripts/stage_native_lib.py``, so the
resulting wheel is NOT pure Python and must carry a platform tag instead of the
default ``py3-none-any`` (otherwise pip would install a Windows wheel on Linux).
"""

from setuptools import setup
from setuptools.dist import Distribution

# bdist_wheel moved from the `wheel` package into setuptools; support both.
try:
    from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
except ImportError:  # pragma: no cover - older setuptools
    from wheel.bdist_wheel import bdist_wheel as _bdist_wheel


class BinaryDistribution(Distribution):
    """Force a platform-specific wheel because a native library is bundled."""

    def has_ext_modules(self) -> bool:  # noqa: D401 - setuptools hook
        return True


class bdist_wheel(_bdist_wheel):
    def finalize_options(self) -> None:
        super().finalize_options()
        # Contains a native library -> not a pure-Python wheel. Keep the Python
        # tag broad (py3) since the library is loaded via ctypes and is
        # independent of the CPython ABI.
        self.root_is_pure = False

    def get_tag(self):
        python, abi, plat = super().get_tag()
        return "py3", "none", plat


setup(distribution_class=BinaryDistribution, cmdclass={"bdist_wheel": bdist_wheel})
