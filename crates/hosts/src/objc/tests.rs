use expect_test::expect;

use crate::tests::check;

#[test]
fn objc_prose_is_c_plus_classes_selectors_and_properties() {
    check(
        "/Sources/Greeter.m",
        r##"/** Greets. */
#import <Foundation/Foundation.h>
#import "Greeter.h"
#define GREET_MAX 4

/// A greeter.
@protocol Greeting <NSObject>
- (void)greet:(NSString *)person times:(NSInteger)times;
@optional
@property (nonatomic, copy) NSString *title;
@end

@interface Greeter : NSObject <Greeting>
@property (nonatomic, strong) NSString *name;
+ (instancetype)greeterWithName:(NSString *)name;
@end

@interface Greeter ()
@property (nonatomic) int hidden;
@end

@interface Greeter (Extras)
- (void)extra;
@end

@implementation Greeter {
    int _count;
}
@synthesize name = _name;

+ (instancetype)greeterWithName:(NSString *)name {
    // Print it.
    Greeter *g = [[Greeter alloc] init];
    NSString *s = @"Hello\nworld";
    NSLog(@"recieve %@", s);
    char *c = "plain";
    return g;
}

- (void)greet:(NSString *)person times:(NSInteger)times {
    [self greetAll];
}
@end

static NSString *const kGreetKey = @"greetKey";
"##,
        expect![[r#"
            objective-c
            > /** Greets. */
                  ^^^^^^^^^^ doccomment
            > #import <Foundation/Foundation.h>
            > #import "Greeter.h"
            > #define GREET_MAX 4
                      ^^^^^^^^^ declaration
            >
            > /// A greeter.
                  ^^^^^^^^^^ doccomment
            > @protocol Greeting <NSObject>
                        ^^^^^^^^ declaration
            > - (void)greet:(NSString *)person times:(NSInteger)times;
                      ^^^^^ declaration
                                        ^^^^^^ declaration
                                               ^^^^^ declaration
                                                                ^^^^^ declaration
            > @optional
            > @property (nonatomic, copy) NSString *title;
                                                    ^^^^^ declaration
            > @end
            >
            > @interface Greeter : NSObject <Greeting>
                         ^^^^^^^ declaration
            > @property (nonatomic, strong) NSString *name;
                                                      ^^^^ declaration
            > + (instancetype)greeterWithName:(NSString *)name;
                              ^^^^^^^^^^^^^^^ declaration
                                                          ^^^^ declaration
            > @end
            >
            > @interface Greeter ()
                         ^^^^^^^ declaration
            > @property (nonatomic) int hidden;
                                        ^^^^^^ declaration
            > @end
            >
            > @interface Greeter (Extras)
                                  ^^^^^^ declaration
            > - (void)extra;
                      ^^^^^ declaration
            > @end
            >
            > @implementation Greeter {
            >     int _count;
                       ^^^^^ declaration
            > }
            > @synthesize name = _name;
            >
            > + (instancetype)greeterWithName:(NSString *)name {
                                                          ^^^^ declaration
            >     // Print it.
                  ^^^^^^^^^^^^ comment
            >     Greeter *g = [[Greeter alloc] init];
                           ^ declaration
            >     NSString *s = @"Hello\nworld";
                            ^ declaration
                                  ^^^^^ value
                                         ^^^^^ value
            >     NSLog(@"recieve %@", s);
                          ^^^^^^^^^^ string
            >     char *c = "plain";
                        ^ declaration
                             ^^^^^ value
            >     return g;
            > }
            >
            > - (void)greet:(NSString *)person times:(NSInteger)times {
                                        ^^^^^^ declaration
                                                                ^^^^^ declaration
            >     [self greetAll];
            > }
            > @end
            >
            > static NSString *const kGreetKey = @"greetKey";
                                     ^^^^^^^^^ declaration
                                                   ^^^^^^^^ value
        "#]],
    );
}

/// `.h` is both C's and Objective-C's header, and a path cannot say which, so it stays with C.
#[test]
fn a_header_path_stays_with_c_and_a_dot_m_is_objective_c() {
    let hosts = crate::Hosts::builtin();
    let id = |path: &str| hosts.for_path(path).map(|host| host.id());
    assert_eq!(id("/Sources/Greeter.h"), Some("c"));
    assert_eq!(id("/Sources/Greeter.m"), Some("objective-c"));
    assert_eq!(id("/Sources/Greeter.mm"), None);
}

/// C's declarations, which Objective-C keeps: each kind of declarator, and what a macro, a struct,
/// a union and an enum name.
#[test]
fn objc_declares_what_c_does() {
    crate::tests::check(
        "/src/shapes.m",
        r#"#define QZXV_AREA(qzxvw, qzxvh) ((qzxvw) * (qzxvh))
typedef struct qzxvpoint { int qzxvx; int qzxvy : 4; } QzxvPoint;
union qzxvbits { int qzxvword; char qzxvbytes[4]; };
enum qzxvcolor { QzxvRed, QzxvGreen = 2 };
static int qzxvcount = 0, *qzxvptr, (*qzxvfn)(int qzxvarg), qzxvgrid[2][2];
int qzxvarea(int qzxvwidth, const char *qzxvlabel);
"#,
        expect_test::expect![[r#"
            objective-c
            > #define QZXV_AREA(qzxvw, qzxvh) ((qzxvw) * (qzxvh))
                      ^^^^^^^^^ declaration
                                ^^^^^ declaration
                                       ^^^^^ declaration
            > typedef struct qzxvpoint { int qzxvx; int qzxvy : 4; } QzxvPoint;
                             ^^^^^^^^^ declaration
                                             ^^^^^ declaration
                                                        ^^^^^ declaration
                                                                     ^^^^^^^^^ declaration
            > union qzxvbits { int qzxvword; char qzxvbytes[4]; };
                    ^^^^^^^^ declaration
                                   ^^^^^^^^ declaration
                                                  ^^^^^^^^^ declaration
            > enum qzxvcolor { QzxvRed, QzxvGreen = 2 };
                   ^^^^^^^^^ declaration
                               ^^^^^^^ declaration
                                        ^^^^^^^^^ declaration
            > static int qzxvcount = 0, *qzxvptr, (*qzxvfn)(int qzxvarg), qzxvgrid[2][2];
                         ^^^^^^^^^ declaration
                                         ^^^^^^^ declaration
                                                    ^^^^^^ declaration
                                                                ^^^^^^^ declaration
                                                                          ^^^^^^^^ declaration
            > int qzxvarea(int qzxvwidth, const char *qzxvlabel);
                  ^^^^^^^^ declaration
                               ^^^^^^^^^ declaration
                                                      ^^^^^^^^^ declaration
        "#]],
    );
}

/// A name in any script, where the grammar reads one, is declared as a plain ASCII one is.
#[test]
fn objc_declares_names_in_any_script() {
    crate::tests::check(
        "/src/names.m",
        "int résumé = 1;\nstruct точка { int ширина; };\n",
        expect_test::expect![[r#"
            objective-c
            > int résumé = 1;
                  ^^^^^^ declaration
            > struct точка { int ширина; };
                     ^^^^^ declaration
                                 ^^^^^^ declaration
        "#]],
    );
}
