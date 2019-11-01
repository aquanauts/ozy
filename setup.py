from setuptools import setup

setup(
    name='ozy',
    description="A nice way to express which apps are installed, where, and with which version",
    version='0.0.1',
    packages=['ozy'],
    license='private',
    url="http://ozy.aq.tc",
    install_requires=[
        "Click==7.0",
        "requests==2.22.0"
    ],
    entry_points={
        # TODO this needs to be a shim shell script or shim app?
        "console_scripts": [
            "ozy = ozy.__main__:main"
        ]
    },
)
